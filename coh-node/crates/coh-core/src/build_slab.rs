use crate::canon::{EXPECTED_SLAB_SCHEMA_ID, EXPECTED_SLAB_VERSION};
use crate::math::CheckedMath;
use crate::merkle::build_merkle_root;
use crate::types::{
    BuildSlabResult, Decision, MicroReceipt, MicroReceiptWire, RejectCode, SlabReceiptWire,
    SlabSummaryWire,
};
use crate::verify_chain::verify_chain;
use std::convert::TryFrom;

#[must_use]
pub fn build_slab(receipts: Vec<MicroReceiptWire>) -> BuildSlabResult {
    if receipts.is_empty() {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Empty chain provided".to_string(),
            range_start: None,
            range_end: None,
            micro_count: None,
            merkle_root: None,
            output: None,
            slab: None,
        };
    }

    // 1. Verify source chain
    let chain_res = verify_chain(receipts.clone());
    if chain_res.decision == Decision::Reject {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: chain_res.code,
            message: format!("Source chain invalid: {}", chain_res.message),
            range_start: Some(chain_res.first_step_index),
            range_end: Some(chain_res.last_step_index),
            micro_count: Some(receipts.len() as u64),
            merkle_root: None,
            output: None,
            slab: None,
        };
    }

    // 2. Aggregate totals and collect leaves
    let mut total_spend: u128 = 0;
    let mut total_defect: u128 = 0;
    let mut total_authority: u128 = 0;
    let first_wire = receipts.first().unwrap();
    let last_wire = receipts.last().unwrap();

    let mut leaves = Vec::with_capacity(receipts.len());

    for wire in &receipts {
        let r = match MicroReceipt::try_from(wire.clone()) {
            Ok(r) => r,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Wire conversion failed in builder: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };

        total_spend = match total_spend.safe_add(r.metrics.spend) {
            Ok(val) => val,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Total spend overflow: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };
        total_defect = match total_defect.safe_add(r.metrics.defect) {
            Ok(val) => val,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Total defect overflow: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };
        total_authority = match total_authority.safe_add(r.metrics.authority) {
            Ok(val) => val,
            Err(e) => {
                return BuildSlabResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Total authority overflow: {:?}", e),
                    range_start: None,
                    range_end: None,
                    micro_count: None,
                    merkle_root: None,
                    output: None,
                    slab: None,
                }
            }
        };
        leaves.push(r.chain_digest_next);
    }

    let merkle_root = build_merkle_root(&leaves);

    let summary = SlabSummaryWire {
        total_spend: total_spend.to_string(),
        total_defect: total_defect.to_string(),
        total_authority: total_authority.to_string(),
        v_pre_first: first_wire.metrics.v_pre.clone(),
        v_post_last: last_wire.metrics.v_post.clone(),
    };

    let slab = SlabReceiptWire {
        schema_id: EXPECTED_SLAB_SCHEMA_ID.to_string(),
        version: EXPECTED_SLAB_VERSION.to_string(),
        object_id: first_wire.object_id.clone(),
        canon_profile_hash: first_wire.canon_profile_hash.clone(),
        policy_hash: first_wire.policy_hash.clone(),
        range_start: first_wire.step_index,
        range_end: last_wire.step_index,
        micro_count: receipts.len() as u64,
        chain_digest_prev: first_wire.chain_digest_prev.clone(),
        chain_digest_next: last_wire.chain_digest_next.clone(),
        state_hash_first: first_wire.state_hash_prev.clone(),
        state_hash_last: last_wire.state_hash_next.clone(),
        merkle_root: merkle_root.to_hex(),
        summary,
    };

    BuildSlabResult {
        decision: Decision::SlabBuilt,
        code: None,
        message: "Slab built successfully".to_string(),
        range_start: Some(slab.range_start),
        range_end: Some(slab.range_end),
        micro_count: Some(slab.micro_count),
        merkle_root: Some(slab.merkle_root.clone()),
        output: None, // Filled by CLI
        slab: Some(slab),
    }
}
