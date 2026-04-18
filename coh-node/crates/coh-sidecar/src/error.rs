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
            // Local integrity failures -> E001
            RejectCode::RejectSchema
            | RejectCode::RejectCanonProfile
            | RejectCode::RejectNumericParse
            | RejectCode::RejectOverflow
            | RejectCode::RejectIntervalInvalid
            | RejectCode::RejectMissingSignature
            | RejectCode::RejectMissingObjectId
            // Trajectory failures -> E004 (chain/continuity)
            | RejectCode::NoProgressLoop
            | RejectCode::StateCycleDetected
            | RejectCode::RetryBudgetExceeded
            | RejectCode::TemporalDriftDetected
            | RejectCode::TrajectoryCostExceeded
            // Resource/governance -> E001 (malformed input)
            | RejectCode::StepBudgetExceeded
            | RejectCode::TimeBudgetExceeded
            | RejectCode::MemoryBudgetExceeded
            | RejectCode::DepthLimitExceeded => CohErrorCode::E001,

            // Chain/continuity -> E002
            RejectCode::RejectChainDigest | RejectCode::RejectSlabMerkle => CohErrorCode::E002,

            // Policy -> E003
            RejectCode::RejectPolicyViolation
            | RejectCode::RejectSlabSummary
            // Semantic integrity failures (TypeConfusion defense)
            | RejectCode::VacuousZeroReceipt
            | RejectCode::SpendExceedsBalance
            | RejectCode::SemanticTypeViolation
            // Cumulative drift failures (GradientDescent defense)
            | RejectCode::CumulativeDriftDetected => CohErrorCode::E003,

            // Measurement/oplax failures -> E003 (policy-style quantitative violation)
            RejectCode::RejectDissipationViolation | RejectCode::RejectInvalidMapping => CohErrorCode::E003,

            // State link -> E004
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
