use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectCode {
    RejectSchema,
    RejectVersion,
    RejectCanonProfile,
    RejectObjectId,
    RejectPolicyHash,
    RejectChainDigestPrev,
    RejectChainDigestNext,
    RejectStateHashLink,
    RejectNumericParse,
    RejectOverflow,
    RejectMerkleRoot,
    RejectSlabSummary,
    RejectRiskBound,
}
