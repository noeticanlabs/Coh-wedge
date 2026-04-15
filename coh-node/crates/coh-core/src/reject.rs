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
    // Resource/governance failures
    StepBudgetExceeded,
    TimeBudgetExceeded,
    MemoryBudgetExceeded,
    DepthLimitExceeded,
}
