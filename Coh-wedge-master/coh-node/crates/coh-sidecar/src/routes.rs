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

#[derive(Debug, Serialize)]
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

pub async fn health_check() -> impl IntoResponse {
    "COH_V1_OK"
}
