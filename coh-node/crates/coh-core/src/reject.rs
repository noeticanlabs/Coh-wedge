use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RejectCode {
    // Local integrity failures
    RejectSchema,
    RejectCanonProfile,
    RejectChainDigest,
    RejectStateHashLink,
    RejectNumericParse,
    RejectOverflow,
    RejectPolicyViolation,
    RejectSlabSummary,
    RejectSlabMerkle,
    RejectIntervalInvalid,
    RejectMissingSignature,
    RejectMissingObjectId,
    RejectInvalidSignature,
    // Trajectory failures
    NoProgressLoop,
    StateCycleDetected,
    RetryBudgetExceeded,
    TemporalDriftDetected,
    TrajectoryCostExceeded,
    // Semantic integrity failures (TypeConfusion defense — Q2)
    VacuousZeroReceipt,
    SpendExceedsBalance,
    SemanticTypeViolation,
    RejectSemanticExecutionMismatch,
    // Cumulative drift failures (GradientDescent defense — Q1)
    CumulativeDriftDetected,
    // Resource/governance failures
    StepBudgetExceeded,
    TimeBudgetExceeded,
    MemoryBudgetExceeded,
    DepthLimitExceeded,
    // Measurement failures
    RejectDissipationViolation,
    RejectInvalidMapping,
    // GCCP Compute-specific failures (Section 18)
    /// Thermal cap exceeded
    RejectTempCap,
    /// Power cap exceeded
    RejectPowerCap,
    /// Queue capacity exceeded
    RejectQueueCap,
    /// Memory cap exceeded
    RejectMemoryCap,
    /// Defect/slack budget exceeded
    RejectDefectCap,
    /// Budget exhausted
    RejectBudget,
    /// Predictor data is stale
    RejectPredictorStale,
    /// Telemetry data is stale
    RejectTelemetryStale,
    /// Requested route unavailable
    RejectRouteUnavailable,
    /// Policy class mismatch
    RejectPolicyClassMismatch,
}
