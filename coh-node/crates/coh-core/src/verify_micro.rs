use crate::types::{MicroReceiptWire, MicroReceipt, VerifyMicroResult, Decision, RejectCode};
use crate::math::CheckedMath;
use crate::canon::{to_prehash_view, to_canonical_json_bytes, EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION, EXPECTED_CANON_PROFILE_HASH};
use crate::hash::compute_chain_digest;
use std::convert::TryFrom;

pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult {
    // 1. Wire to runtime conversion
    let r = match MicroReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => return VerifyMicroResult { decision: Decision::Reject, code: Some(e) },
    };

    // 2. Schema check
    if r.schema_id != EXPECTED_MICRO_SCHEMA_ID {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectSchema) };
    }
    if r.version != EXPECTED_MICRO_VERSION {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectVersion) };
    }

    // 3. Canon profile check
    if r.canon_profile_hash.to_hex() != EXPECTED_CANON_PROFILE_HASH {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectCanonProfile) };
    }

    // 4. Field sanity
    // (In v1, we mostly rely on successful parsing and basic continuity checks in chain)
    // One specific sanity: object_id must not be empty.
    if r.object_id.is_empty() {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectObjectId) };
    }

    // 5. Checked arithmetic
    // v_post + spend
    let left_side = match r.metrics.v_post.safe_add(r.spend) {
        Ok(val) => val,
        Err(e) => return VerifyMicroResult { decision: Decision::Reject, code: Some(e) },
    };
    // v_pre + defect
    let right_side = match r.metrics.v_pre.safe_add(r.defect) {
        Ok(val) => val,
        Err(e) => return VerifyMicroResult { decision: Decision::Reject, code: Some(e) },
    };

    // 6. Policy inequality
    if left_side > right_side {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectRiskBound) };
    }

    // 7. Digest recomputation
    let prehash = to_prehash_view(&r);
    let canon_bytes = match to_canonical_json_bytes(&prehash) {
        Ok(b) => b,
        Err(e) => return VerifyMicroResult { decision: Decision::Reject, code: Some(e) },
    };
    let recomputed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);

    // 8. Digest compare
    if r.chain_digest_next != recomputed_digest {
        return VerifyMicroResult { decision: Decision::Reject, code: Some(RejectCode::RejectChainDigestNext) };
    }

    VerifyMicroResult { decision: Decision::Accept, code: None }
}
