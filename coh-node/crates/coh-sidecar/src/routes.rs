use crate::error::{ApiError, CohErrorCode};
use axum::{response::IntoResponse, Json};
use coh_core::types::{Decision, MicroReceiptWire};
use coh_core::{verify_chain, verify_micro};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct VerifyChainRequest {
    pub receipts: Vec<MicroReceiptWire>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedResponse<T> {
    pub request_id: String,
    pub coh_version: String,
    pub status: Decision,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
pub struct ChainBreakDetails {
    pub break_index: u64,
    pub message: String,
}

pub async fn verify_micro_handler(Json(payload): Json<MicroReceiptWire>) -> impl IntoResponse {
    let request_id = Uuid::new_v4().to_string();
    let result = verify_micro(payload);

    let mut error = None;
    if result.decision != Decision::Accept {
        error = Some(ApiError {
            code: result.code.map(|c| c.into()).unwrap_or(CohErrorCode::E001),
            message: result.message,
            request_id: request_id.clone(),
        });
    }

    Json(UnifiedResponse {
        request_id,
        coh_version: "0.1.0".to_string(),
        status: result.decision,
        data: None::<serde_json::Value>,
        error,
    })
}

pub async fn verify_chain_handler(Json(payload): Json<VerifyChainRequest>) -> impl IntoResponse {
    let request_id = Uuid::new_v4().to_string();
    let result = verify_chain(payload.receipts);

    let mut error = None;
    let mut data = None;

    if result.decision != Decision::Accept {
        error = Some(ApiError {
            code: result.code.map(|c| c.into()).unwrap_or(CohErrorCode::E004),
            message: result.message.clone(),
            request_id: request_id.clone(),
        });

        data = Some(ChainBreakDetails {
            break_index: result.failing_step_index.unwrap_or(0),
            message: result.message,
        });
    }

    Json(UnifiedResponse {
        request_id,
        coh_version: "0.1.0".to_string(),
        status: result.decision,
        data,
        error,
    })
}

#[derive(Debug, Deserialize)]
pub struct ExecuteVerifiedRequest {
    pub receipt: MicroReceiptWire,
    pub action: serde_json::Value,
}

pub async fn execute_verified_handler(
    Json(payload): Json<ExecuteVerifiedRequest>,
) -> impl IntoResponse {
    let request_id = Uuid::new_v4().to_string();
    let result = verify_micro(payload.receipt);

    if result.decision != Decision::Accept {
        let req_id = request_id.clone();
        return Json(UnifiedResponse {
            request_id,
            coh_version: "0.1.0".to_string(),
            status: result.decision,
            data: None::<serde_json::Value>,
            error: Some(ApiError {
                code: result.code.map(|c| c.into()).unwrap_or(CohErrorCode::E003),
                message: format!("Execution blocked: {}", result.message),
                request_id: req_id,
            }),
        });
    }

    Json(UnifiedResponse {
        request_id,
        coh_version: "0.1.0".to_string(),
        status: Decision::Accept,
        data: Some(payload.action),
        error: None,
    })
}

use coh_core::trajectory::engine::{search, SearchContext};
use coh_core::trajectory::search_result::SearchResult;
use tokio::time::{timeout, Duration};

const MAX_BEAM: usize = 8;
const MAX_DEPTH: usize = 6;
const SEARCH_TIMEOUT_MS: u64 = 500;

#[derive(Debug, Deserialize, Serialize)]
pub struct TrajectorySearchRequest {
    pub context: SearchContext,
}

pub async fn trajectory_search_handler(
    Json(payload): Json<TrajectorySearchRequest>,
) -> impl IntoResponse {
    let request_id = Uuid::new_v4().to_string();

    // 1. Budget Guards
    if payload.context.beam_width > MAX_BEAM || payload.context.max_depth > MAX_DEPTH {
        return Json(UnifiedResponse {
            request_id: request_id.clone(),
            coh_version: "0.1.0".to_string(),
            status: Decision::Reject,
            data: None::<SearchResult>,
            error: Some(ApiError {
                code: CohErrorCode::E003,
                message: format!(
                    "Search budget exceeded. Max Beam: {}, Max Depth: {}",
                    MAX_BEAM, MAX_DEPTH
                ),
                request_id,
            }),
        });
    }

    // 2. Execution with Timeout
    let search_task = async { search(&payload.context) };
    let result = timeout(Duration::from_millis(SEARCH_TIMEOUT_MS), search_task).await;

    match result {
        Ok(search_result) => Json(UnifiedResponse {
            request_id,
            coh_version: "0.1.0".to_string(),
            status: Decision::Accept,
            data: Some(search_result),
            error: None,
        }),
        Err(_) => Json(UnifiedResponse {
            request_id: request_id.clone(),
            coh_version: "0.1.0".to_string(),
            status: Decision::Reject,
            data: None::<SearchResult>,
            error: Some(ApiError {
                code: CohErrorCode::E003,
                message: format!("Search timed out after {}ms", SEARCH_TIMEOUT_MS),
                request_id,
            }),
        }),
    }
}

pub async fn health_check() -> impl IntoResponse {
    "COH_V1_OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use coh_core::trajectory::{DomainState, FinancialState, FinancialStatus, ScoringWeights, SearchContext};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;
    use axum::Router;
    use axum::routing::post;
    use http_body_util::BodyExt; // For collect()

    fn test_app() -> Router {
        Router::new().route("/trajectory/search", post(trajectory_search_handler))
    }

    #[tokio::test]
    async fn test_trajectory_search_budget_guard() {
        let app = test_app();
        
        // Context exceeding budget
        let idle_f = FinancialState {
            balance: 1000,
            initial_balance: 1000,
            status: FinancialStatus::Idle,
            current_invoice_amount: 0,
        };
        let context = SearchContext {
            initial_state: DomainState::Financial(idle_f.clone()),
            target_state: DomainState::Financial(idle_f),
            max_depth: 10, // MAX is 6
            beam_width: 10, // MAX is 8
            weights: ScoringWeights::default(),
        };
        let req_payload = TrajectorySearchRequest { context };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/trajectory/search")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&req_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let res: UnifiedResponse<SearchResult> = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(res.status, Decision::Reject);
        assert!(res.error.unwrap().message.contains("Search budget exceeded"));
    }
}
