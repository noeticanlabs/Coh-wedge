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
    // Cumulative drift failures (GradientDescent defense — Q1)
    CumulativeDriftDetected,
    // Resource/governance failures
    StepBudgetExceeded,
    TimeBudgetExceeded,
    MemoryBudgetExceeded,
    DepthLimitExceeded,
}
