//! LLM Proxy module — OpenAI-compatible proxy with inline receipt verification

// Inline session module
mod session {
    #[derive(Debug, Clone)]
    pub struct SessionState {
        pub step_index: u64,
        pub state_hash: String,
        pub chain_digest: String,
        pub budget_remaining: u128,
    }
    impl SessionState {
        pub fn new(initial_budget: u128) -> Self {
            Self {
                step_index: 0,
                state_hash: "0".repeat(64),
                chain_digest: "0".repeat(64),
                budget_remaining: initial_budget,
            }
        }
    }
}

// Inline receipt module
mod receipt {
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view, EXPECTED_CANON_PROFILE_HASH};
    use coh_core::hash::compute_chain_digest;
    use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire, SignatureWire};
    use std::convert::TryFrom;
    const POLICY_HASH_ZERO: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        step_index: u64,
        state_hash_prev: &str,
        chain_digest_prev: &str,
        v_pre: u128,
        v_post: u128,
        spend: u128,
        defect: u128,
        authority: u128,
        model: &str,
    ) -> MicroReceiptWire {
        let object_id = format!("llm.{}.{}", model.replace(['/', '-', '.'], "_"), step_index);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock moved backwards")
            .as_secs();
        MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id,
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: POLICY_HASH_ZERO.to_string(),
            step_index,
            step_type: Some("llm_chat".to_string()),
            signatures: Some(vec![SignatureWire {
                signature: format!("sig-{:016}", step_index),
                signer: "coh-sidecar".to_string(),
                timestamp: now,
                authority_id: Some("coh-sidecar".to_string()),
                scope: Some("*".to_string()),
                expires_at: None,
            }]),
            state_hash_prev: state_hash_prev.to_string(),
            state_hash_next: format!("{:064x}", v_post),
            chain_digest_prev: chain_digest_prev.to_string(),
            chain_digest_next: POLICY_HASH_ZERO.to_string(),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: defect.to_string(),
                authority: authority.to_string(),
            },
        }
    }
    pub fn reseal_in_place(wire: &mut MicroReceiptWire) -> Result<(), String> {
        let runtime = MicroReceipt::try_from(wire.clone()).map_err(|e| format!("{:?}", e))?;
        let prehash = to_prehash_view(&runtime);
        let bytes = to_canonical_json_bytes(&prehash).map_err(|e| format!("{:?}", e))?;
        wire.chain_digest_next = compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex();
        Ok(())
    }
}

use axum::{
    extract::State, http::HeaderMap, http::StatusCode, response::IntoResponse, response::Response,
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Shared application state containing all session states and a shared HTTP client
pub struct AppState {
    pub sessions: Arc<Mutex<std::collections::HashMap<String, session::SessionState>>>,
    pub http_client: Client,
    pub upstream_url: String,
    pub metrics: Arc<Mutex<Metrics>>,
}

/// Running metrics for the proxy
#[derive(Debug, Default)]
pub struct Metrics {
    pub total_requests: u64,
    pub accepts: u64,
    pub rejects: u64,
    pub latency_nanos_sum: u64,
    pub latency_count: u64,
}

impl AppState {
    pub fn new(upstream_url: String) -> anyhow::Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to build reqwest client for AppState")?;

        Ok(Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            http_client,
            upstream_url,
            metrics: Arc::new(Mutex::new(Metrics::default())),
        })
    }
}

/// OpenAI chat completion request (passthrough)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// OpenAI chat completion usage
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI chat completion choice
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

/// OpenAI chat completion response (passthrough)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

/// Unified wrapper around an LLM response, optionally including the Coh receipt and decision
#[derive(Debug, Serialize)]
pub struct LlmResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coh_receipt: Option<coh_core::types::MicroReceiptWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coh_decision: Option<String>,
}

/// Error response from the proxy
#[derive(Debug, Serialize)]
pub struct ProxyError {
    pub error: ProxyErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ProxyErrorDetail {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coh_decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coh_receipt: Option<coh_core::types::MicroReceiptWire>,
}

impl ProxyError {
    pub fn new(message: String, code: Option<String>) -> Self {
        Self {
            error: ProxyErrorDetail {
                message,
                r#type: "coh_verification_error".to_string(),
                code,
                coh_decision: Some("REJECT".to_string()),
                coh_receipt: None,
            },
        }
    }

    pub fn with_receipt(mut self, receipt: coh_core::types::MicroReceiptWire) -> Self {
        self.error.coh_receipt = Some(receipt);
        self
    }
}

/// Handle incoming chat completion requests — forward to upstream, generate receipt, verify, return
pub async fn chat_completions_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<ChatCompletionRequest>,
) -> Response {
    let _request_id = Uuid::new_v4().to_string();
    let start = Instant::now();
    let session_id = headers
        .get("x-session-token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("default")
        .to_string();

    // Extract token budget from header or default
    let budget = headers
        .get("x-coh-budget")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    // Forward to upstream using reqwest client directly
    let client = &state.http_client;
    let body = serde_json::to_vec(&payload).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ProxyError::new(format!("Serialization failed: {}", e), None)),
        )
            .into_response()
    })?;

    let upstream_response = match client
        .post(format!("{}/chat/completions", state.upstream_url))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // If upstream unreachable, return 503
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ProxyError::new(
                    format!("Upstream unavailable: {}", e),
                    None,
                )),
            )
                .into_response();
        }
    };

    let status = upstream_response.status();
    if !status.is_success() {
        // Pass through upstream errors directly (e.g., rate limits)
        let body = upstream_response.text().await.unwrap_or_default();
        return (
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            body,
        )
            .into_response();
    }

    // Parse upstream response
    let llm_response: ChatCompletionResponse = match upstream_response.json().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProxyError::new(
                    format!("Invalid upstream JSON: {}", e),
                    None,
                )),
            )
                .into_response();
        }
    };

    // Determine metrics from usage
    let total_tokens = llm_response.usage.total_tokens;
    let _completion_tokens = llm_response.usage.completion_tokens;
    let _prompt_tokens = llm_response.usage.prompt_tokens;
    let spend = total_tokens as u128;
    let v_pre = budget as u128;
    let v_post = (v_pre as i128 - spend as i128).max(0) as u128;

    // Map finish_reason to authority/defect
    let is_terminal = llm_response
        .choices
        .first()
        .map(|c| c.finish_reason == "stop")
        .unwrap_or(false);
    let authority = if is_terminal { 1u128 } else { 0u128 };
    let defect = if is_terminal { 0u128 } else { 1u128 };

    // Get session state
    let (step_index, state_hash_prev, chain_digest_prev) = {
        let mut sessions = state.sessions.lock().await;
        let session = sessions
            .entry(session_id.clone())
            .or_insert_with(|| session::SessionState::new(budget as u128));
        let si = session.step_index;
        let sh = session.state_hash.clone();
        let cd = session.chain_digest.clone();
        session.step_index += 1;
        (si, sh, cd)
    };

    // Generate receipt
    let mut receipt = receipt::build(
        step_index,
        &state_hash_prev,
        &chain_digest_prev,
        v_pre,
        v_post,
        spend,
        defect,
        authority,
        &llm_response.model,
    );

    // Seal the receipt
    if let Err(e) = receipt::reseal_in_place(&mut receipt) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProxyError::new(
                format!("Receipt sealing failed: {}", e),
                None,
            )),
        )
            .into_response();
    }

    // Verify the receipt
    let verify_result = coh_core::verify_micro(receipt.clone());

    // Update metrics
    {
        let mut m = state.metrics.lock().await;
        m.total_requests += 1;
        let elapsed = start.elapsed().as_nanos() as u64;
        m.latency_nanos_sum += elapsed;
        m.latency_count += 1;
    }

    // Decide: ACCEPT or REJECT
    match verify_result.decision {
        coh_core::types::Decision::Accept => {
            // ACCEPT: update session state
            {
                let mut sessions = state.sessions.lock().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.state_hash = receipt.state_hash_next.clone();
                    session.chain_digest = receipt.chain_digest_next.clone();
                    session.budget_remaining = v_post;
                }
            }

            // Update metrics
            {
                let mut m = state.metrics.lock().await;
                m.accepts += 1;
            }

            // Return response with integrated receipt
            let wrapper = LlmResponse {
                id: llm_response.id,
                object: llm_response.object,
                created: llm_response.created,
                model: llm_response.model,
                choices: llm_response.choices,
                usage: llm_response.usage,
                coh_receipt: Some(receipt),
                coh_decision: Some("ACCEPT".to_string()),
            };
            Json(wrapper).into_response()
        }
        _ => {
            // REJECT: block the response
            {
                let mut m = state.metrics.lock().await;
                m.rejects += 1;
            }

            let wrapper = ProxyError::new(
                format!("Coh verification failed at step {}. Token spend ({}) exceeds budget ({}). Receipt rejected.", step_index, spend, budget),
                verify_result.code.map(|c| format!("{:?}", c)),
            )
            .with_receipt(receipt);

            (StatusCode::FORBIDDEN, Json(wrapper)).into_response()
        }
    }
}

/// GET /coh/stats — return running accept/reject counts and latency percentiles
pub async fn coh_stats_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let m = state.metrics.lock().await;

    // Compute percentiles in a real impl would need storing latencies; here we use simple averages
    let avg_latency_ms = if m.latency_count > 0 {
        (m.latency_nanos_sum / m.latency_count) as f64 / 1_000_000.0
    } else {
        0.0
    };

    let acceptance_rate = if m.total_requests > 0 {
        (m.accepts as f64 / m.total_requests as f64) * 100.0
    } else {
        0.0
    };

    Json(serde_json::json!({
        "total_requests": m.total_requests,
        "accepts": m.accepts,
        "rejects": m.rejects,
        "false_accept_rate": 0.0, // not trackable without ground truth
        "false_reject_rate": 0.0, // not trackable without ground truth
        "avg_latency_ms": avg_latency_ms,
        "acceptance_rate_%": acceptance_rate,
    }))
}

/// POST /coh/reset — reset session state
pub async fn coh_reset_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let mut sessions = state.sessions.lock().await;
    sessions.remove(&session_id);

    axum::Json(serde_json::json!({ "status": "reset", "session_id": session_id }))
}

/// GET /v1/models — OpenAI-compatible models list
pub async fn models_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "coh-proxy",
                "object": "model",
                "created": 1700000000,
                "owned_by": "coh-sidecar"
            }
        ]
    }))
}

/// GET /coh/chain — return current chain digest and step count for a session
pub async fn coh_chain_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::Json<serde_json::Value> {
    let session_id = params
        .get("session_id")
        .map(|s| s.as_str())
        .unwrap_or("default");
    let sessions = state.sessions.lock().await;

    if let Some(session) = sessions.get(session_id) {
        axum::Json(serde_json::json!({
            "session_id": session_id,
            "step_index": session.step_index,
            "chain_digest": session.chain_digest,
            "budget_remaining": session.budget_remaining
        }))
    } else {
        axum::Json(serde_json::json!({
            "session_id": session_id,
            "step_index": 0,
            "chain_digest": "0".repeat(64),
            "budget_remaining": 1000
        }))
    }
}
