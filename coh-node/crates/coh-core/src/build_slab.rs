use crate::types::{MicroReceiptWire, MicroReceipt, SlabReceiptWire, SlabSummaryWire, BuildSlabResult, Decision, RejectCode};
use crate::math::CheckedMath;
use crate::merkle::build_merkle_root;
use crate::verify_chain::verify_chain;
use std::convert::TryFrom;

pub fn build_slab(receipts: Vec<MicroReceiptWire>) -> BuildSlabResult {
    if receipts.is_empty() {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            slab: None,
        };
    }

    // 1. Verify source chain
    let chain_res = verify_chain(receipts.clone());
    if chain_res.decision == Decision::Reject {
        return BuildSlabResult {
            decision: Decision::Reject,
            code: chain_res.code,
            slab: None,
        };
    }

    // 2. Aggregate totals and collect leaves
    let mut total_spend: u128 = 0;
    let mut total_defect: u128 = 0;
    let first_wire = receipts.first().unwrap();
    let last_wire = receipts.last().unwrap();
    
    let mut leaves = Vec::with_capacity(receipts.len());

    for wire in &receipts {
        let r = match MicroReceipt::try_from(wire.clone()) {
            Ok(r) => r,
            Err(e) => return BuildSlabResult { decision: Decision::Reject, code: Some(e), slab: None },
        };
        
        total_spend = match total_spend.safe_add(r.spend) {
            Ok(val) => val,
            Err(e) => return BuildSlabResult { decision: Decision::Reject, code: Some(e), slab: None },
        };
        total_defect = match total_defect.safe_add(r.defect) {
            Ok(val) => val,
            Err(e) => return BuildSlabResult { decision: Decision::Reject, code: Some(e), slab: None },
        };
        leaves.push(r.chain_digest_next);
    }

    let merkle_root = build_merkle_root(&leaves);

    let summary = SlabSummaryWire {
        state_hash_pre: first_wire.state_hash_prev.clone(),
        state_hash_post: last_wire.state_hash_next.clone(),
        v_pre: first_wire.metrics.v_pre.clone(),
        v_post: last_wire.metrics.v_post.clone(),
        spend: total_spend.to_string(),
        defect: total_defect.to_string(),
    };

    let slab = SlabReceiptWire {
        schema_id: "coh.slab.v1".to_string(),
        version: 1,
        object_id: first_wire.object_id.clone(),
        canon_profile_hash: first_wire.canon_profile_hash.clone(),
        policy_hash: first_wire.policy_hash.clone(),
        range_start: first_wire.step_index,
        range_end: last_wire.step_index,
        micro_count: receipts.len() as u64,
        chain_digest_prev: first_wire.chain_digest_prev.clone(),
        chain_digest_next: last_wire.chain_digest_next.clone(),
        merkle_root: merkle_root.to_hex(),
        summary,
    };

    BuildSlabResult {
        decision: Decision::Accept,
        code: None,
        slab: Some(slab),
    }
}
