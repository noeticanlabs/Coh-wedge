use crate::canon::{
    to_canonical_json_bytes, to_prehash_view, EXPECTED_CANON_PROFILE_HASH,
    EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION,
};
use crate::hash::compute_chain_digest;
use crate::math::CheckedMath;
use crate::types::{Decision, MicroReceipt, MicroReceiptWire, RejectCode, VerifyMicroResult};
use std::convert::TryFrom;

#[must_use]
pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult {
    let step_index = wire.step_index;
    let object_id = wire.object_id.clone();

    // 1. Wire to runtime (handles hex validation and numeric parsing)
    let r = match MicroReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Malformed input: invalid hex or numeric format".to_string(),
                step_index: Some(step_index),
                object_id: Some(object_id),
                chain_digest_next: None,
            }
        }
    };

    // 2. Schema check
    if r.schema_id != EXPECTED_MICRO_SCHEMA_ID {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Invalid schema_id: {} (Expected: {})",
                r.schema_id, EXPECTED_MICRO_SCHEMA_ID
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }
    if r.version != EXPECTED_MICRO_VERSION {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Unsupported version: {} (Expected: {})",
                r.version, EXPECTED_MICRO_VERSION
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 3. Object ID sanity
    if r.object_id.trim().is_empty() {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Missing object_id".to_string(),
            step_index: Some(r.step_index),
            object_id: Some("".to_string()),
            chain_digest_next: None,
        };
    }

    // 4. Profile check
    if r.canon_profile_hash.to_hex() != EXPECTED_CANON_PROFILE_HASH {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectCanonProfile),
            message: format!(
                "Canonical profile mismatch: {} (Expected: {})",
                r.canon_profile_hash.to_hex(),
                EXPECTED_CANON_PROFILE_HASH
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 5. Policy logic (Arithmetic boundary check)
    // Constraint: v_post + spend <= v_pre + defect
    let lhs = match r.metrics.v_post.safe_add(r.metrics.spend) {
        Ok(val) => val,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Policy arithmetic overflow (v_post + spend)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: None,
            }
        }
    };
    let rhs = match r.metrics.v_pre.safe_add(r.metrics.defect) {
        Ok(val) => val,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Policy arithmetic overflow (v_pre + defect)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: None,
            }
        }
    };

    if lhs > rhs {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: format!(
                "Policy violation: v_post + spend ({}) exceeds v_pre + defect ({})",
                lhs, rhs
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 6. Cryptographic integrity (Canonicalization + Hashing)
    let prehash = to_prehash_view(&r);
    let canon_bytes = to_canonical_json_bytes(&prehash).unwrap();
    let computed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);

    if computed_digest != r.chain_digest_next {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectChainDigest),
            message: format!(
                "Cryptographic digest mismatch: computed {} but found {}",
                computed_digest.to_hex(),
                r.chain_digest_next.to_hex()
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: Some(r.chain_digest_next.to_hex()),
        };
    }

    VerifyMicroResult {
        decision: Decision::Accept,
        code: None,
        message: "Micro-receipt verified successfully".to_string(),
        step_index: Some(r.step_index),
        object_id: Some(r.object_id),
        chain_digest_next: Some(r.chain_digest_next.to_hex()),
    }
}
