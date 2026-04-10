use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use coh_core::reject::RejectCode;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy)]
pub enum CohErrorCode {
    E001, // Malformed Input / Schema / Overflow
    E002, // Cryptographic Failure (Digest/Merkle)
    E003, // Policy Violation (Accounting Law)
    E004, // Chain Discontinuity (State linkage)
}

impl From<RejectCode> for CohErrorCode {
    fn from(code: RejectCode) -> Self {
        match code {
            RejectCode::RejectSchema
            | RejectCode::RejectCanonProfile
            | RejectCode::RejectNumericParse
            | RejectCode::RejectOverflow => CohErrorCode::E001,

            RejectCode::RejectChainDigest | RejectCode::RejectSlabMerkle => CohErrorCode::E002,

            RejectCode::RejectPolicyViolation | RejectCode::RejectSlabSummary => CohErrorCode::E003,

            RejectCode::RejectStateHashLink => CohErrorCode::E004,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: CohErrorCode,
    pub message: String,
    pub request_id: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
