use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RejectCode {
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
}
