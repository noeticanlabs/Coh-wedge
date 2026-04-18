use crate::canon::{EXPECTED_SLAB_SCHEMA_ID, EXPECTED_SLAB_VERSION};
use crate::math::CheckedMath;
use crate::merkle;
use crate::types::{Decision, RejectCode, SlabReceipt, SlabReceiptWire, VerifySlabResult};
use std::convert::TryFrom;

/// NOTE: This verifies macro-accounting integrity but does NOT verify the Merkle root.
/// Full Merkle verification requires `verify_slab_with_leaves()`.
/// - `verify_slab_envelope()` = summary/envelope verification only
/// - `verify_slab_with_leaves()` = full merkle verification
#[must_use]
pub fn verify_slab_envelope(wire: SlabReceiptWire) -> VerifySlabResult {
    let r = match SlabReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: format!("Wire conversion failed: {:?}", e),
                range_start: 0,
                range_end: 0,
                micro_count: None,
                merkle_root: None,
            }
        }
    };

    if r.schema_id != EXPECTED_SLAB_SCHEMA_ID {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Invalid schema_id: {} (Expected: {})",
                r.schema_id, EXPECTED_SLAB_SCHEMA_ID
            ),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    if r.version != EXPECTED_SLAB_VERSION {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Unsupported version: {} (Expected: {})",
                r.version, EXPECTED_SLAB_VERSION
            ),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    if r.micro_count == 0 {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: "Slab is empty (micro_count = 0).".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(0),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }
    if r.range_end < r.range_start {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: "Invalid range.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    let expected_count = r.range_end - r.range_start + 1;
    if expected_count != r.micro_count {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: "Range count mismatch.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    let left_side = match r.summary.v_post_last.safe_add(r.summary.total_spend) {
        Ok(val) => val,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Overflow".to_string(),
                range_start: r.range_start,
                range_end: r.range_end,
                micro_count: Some(r.micro_count),
                merkle_root: Some(r.merkle_root.to_hex()),
            }
        }
    };
    let right_side = match r.summary.v_pre_first.safe_add(r.summary.total_defect) {
        Ok(val) => match val.safe_add(r.summary.total_authority) {
            Ok(total) => total,
            Err(e) => {
                return VerifySlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: "Overflow (v_pre + defect + authority)".to_string(),
                    range_start: r.range_start,
                    range_end: r.range_end,
                    micro_count: Some(r.micro_count),
                    merkle_root: Some(r.merkle_root.to_hex()),
                }
            }
        },
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Overflow".to_string(),
                range_start: r.range_start,
                range_end: r.range_end,
                micro_count: Some(r.micro_count),
                merkle_root: Some(r.merkle_root.to_hex()),
            }
        }
    };

    if left_side > right_side {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: "Macro inequality violated.".to_string(),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    VerifySlabResult {
        decision: Decision::Accept,
        code: None,
        message: "Slab verified successfully.".to_string(),
        range_start: r.range_start,
        range_end: r.range_end,
        micro_count: Some(r.micro_count),
        merkle_root: Some(r.merkle_root.to_hex()),
    }
}

#[must_use]
pub fn verify_slab_with_leaves(
    wire: SlabReceiptWire,
    leaves: Vec<crate::types::Hash32>,
) -> VerifySlabResult {
    let wire_clone = wire.clone();
    let mut result = verify_slab_envelope(wire);
    if result.decision != Decision::Accept {
        return result;
    }

    let slab = crate::types::SlabReceipt::try_from(wire_clone).unwrap();
    match merkle::verify_merkle_root(slab.merkle_root, &leaves) {
        Ok(()) => {
            result.message = "Slab verified successfully including Merkle root.".to_string();
            result
        }
        Err(()) => VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabMerkle),
            message: "Merkle root mismatch.".to_string(),
            range_start: slab.range_start,
            range_end: slab.range_end,
            micro_count: Some(slab.micro_count),
            merkle_root: Some(slab.merkle_root.to_hex()),
        },
    }
}
