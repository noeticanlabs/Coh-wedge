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
  deriving Repr, DecidableEq

end Coh.Contract
