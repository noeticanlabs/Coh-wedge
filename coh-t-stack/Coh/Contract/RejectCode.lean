import Coh.Prelude

namespace Coh.Contract

inductive RejectCode where
  | rejectSchema
  | rejectCanonProfile
  | rejectChainDigest
  | rejectStateHashLink
  | rejectNumericParse
  | rejectOverflow
  | rejectPolicyViolation
  | rejectSlabSummary
  | rejectSlabMerkle
  | rejectIntervalInvalid
  | rejectMissingSignature
  | rejectMissingObjectId
  | noProgressLoop
  | stateCycleDetected
  | retryBudgetExceeded
  | temporalDriftDetected
  | trajectoryCostExceeded
  | vacuousZeroReceipt
  | spendExceedsBalance
  | semanticTypeViolation
  | cumulativeDriftDetected
  | stepBudgetExceeded
  | timeBudgetExceeded
  | memoryBudgetExceeded
  | depthLimitExceeded
  deriving Repr, DecidableEq

end Coh.Contract
