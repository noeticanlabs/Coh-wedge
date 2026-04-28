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
    E007,
    E008,
    E009,
    E010,
    E011,
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
            | RejectCode::RejectSignatureMalformed
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
            | RejectCode::RejectSignatureBad
            | RejectCode::RejectSignerUnknown
            | RejectCode::RejectSignerUntrusted
            | RejectCode::RejectSignatureScopeMismatch
            | RejectCode::RejectSignaturePolicyMismatch
            | RejectCode::RejectSignatureExpired
            // Semantic integrity failures (TypeConfusion defense)
            | RejectCode::VacuousZeroReceipt
            | RejectCode::SpendExceedsBalance
            | RejectCode::SemanticTypeViolation
            // Cumulative drift failures (GradientDescent defense)
            | RejectCode::CumulativeDriftDetected => CohErrorCode::E003,

            // State link -> E004
            RejectCode::RejectStateHashLink => CohErrorCode::E004,
            RejectCode::ChaosMissing => CohErrorCode::E007,
            RejectCode::ChaosViolation => CohErrorCode::E008,
            RejectCode::ProjectionMismatch => CohErrorCode::E009,
            RejectCode::SemanticEnvelopeMissing => CohErrorCode::E010,
            RejectCode::SemanticEnvelopeViolation => CohErrorCode::E011,
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
