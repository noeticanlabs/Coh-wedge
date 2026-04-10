use crate::canon::{EXPECTED_SLAB_SCHEMA_ID, EXPECTED_SLAB_VERSION};
use crate::math::CheckedMath;
use crate::merkle;
use crate::types::{Decision, RejectCode, SlabReceipt, SlabReceiptWire, VerifySlabResult};
use std::convert::TryFrom;

/// Standalone slab verification (v1 mode).
///
/// NOTE: This verifies macro-accounting integrity but does NOT verify the Merkle root.
/// Full Merkle verification requires access to the original chain digests
/// (the leaves). Use `verify_slab_with_leaves()` for complete verification.
#[must_use]
pub fn verify_slab(wire: SlabReceiptWire) -> VerifySlabResult {
    // 1. Wire to runtime
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

    // 2. Envelope checks
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

    // 3. Range sanity (Slab Summary layer)
    if r.micro_count == 0 {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message:
                "Slab is empty (micro_count = 0). Slab must contain at least one micro-receipt."
                    .to_string(),
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
            message: format!(
                "Invalid range: {}..{} (End index cannot be less than start index)",
                r.range_start, r.range_end
            ),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    // Exactly count matches interval check:
    let expected_count = r.range_end - r.range_start + 1;
    if expected_count != r.micro_count {
        return VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabSummary),
            message: format!(
                "Range count mismatch: interval {}..{} implies {} steps, but micro_count is {}",
                r.range_start, r.range_end, expected_count, r.micro_count
            ),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    // 4. Macro inequality
    let left_side = match r.summary.v_post_last.safe_add(r.summary.total_spend) {
        Ok(val) => val,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: format!(
                    "Macro arithmetic overflow (v_post_last + total_spend): {:?}",
                    e
                ),
                range_start: r.range_start,
                range_end: r.range_end,
                micro_count: Some(r.micro_count),
                merkle_root: Some(r.merkle_root.to_hex()),
            }
        }
    };
    let right_side = match r.summary.v_pre_first.safe_add(r.summary.total_defect) {
        Ok(val) => val,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: format!(
                    "Macro arithmetic overflow (v_pre_first + total_defect): {:?}",
                    e
                ),
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
            message: format!("Macro inequality violated: v_post_last + total_spend ({}) exceeds v_pre_first + total_defect ({})", left_side, right_side),
            range_start: r.range_start,
            range_end: r.range_end,
            micro_count: Some(r.micro_count),
            merkle_root: Some(r.merkle_root.to_hex()),
        };
    }

    VerifySlabResult {
        decision: Decision::Accept,
        code: None,
        message: "Slab verified successfully: range checked and macro-accounting balanced."
            .to_string(),
        range_start: r.range_start,
        range_end: r.range_end,
        micro_count: Some(r.micro_count),
        merkle_root: Some(r.merkle_root.to_hex()),
    }
}

/// Full slab verification including Merkle root verification.
///
/// Takes the slab wire and the original chain digests (leaves) to compute
/// and verify the Merkle root.
#[must_use]
pub fn verify_slab_with_leaves(
    wire: SlabReceiptWire,
    leaves: Vec<crate::types::Hash32>,
) -> VerifySlabResult {
    // Extract what we need before moving wire
    let wire_clone = wire.clone();

    // First run standard verification for schema, range, and macro policy
    let mut result = verify_slab(wire);

    // If already rejected, return early
    if result.decision != Decision::Accept {
        return result;
    }

    // Now verify Merkle root
    let slab = match crate::types::SlabReceipt::try_from(wire_clone) {
        Ok(s) => s,
        Err(e) => {
            return VerifySlabResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Wire conversion failed for Merkle verification".to_string(),
                range_start: 0,
                range_end: 0,
                micro_count: None,
                merkle_root: None,
            }
        }
    };

    // Verify Merkle root matches computed root from leaves
    match merkle::verify_merkle_root(slab.merkle_root, &leaves) {
        Ok(()) => {
            result.message =
                "Slab verified successfully: range, macro-accounting, and Merkle root all valid."
                    .to_string();
            result
        }
        Err(()) => VerifySlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSlabMerkle),
            message: format!(
                "Merkle root mismatch: expected {}, computed {}",
                slab.merkle_root.to_hex(),
                crate::merkle::build_merkle_root(&leaves).to_hex()
            ),
            range_start: slab.range_start,
            range_end: slab.range_end,
            micro_count: Some(slab.micro_count),
            merkle_root: Some(slab.merkle_root.to_hex()),
        },
    }
}
