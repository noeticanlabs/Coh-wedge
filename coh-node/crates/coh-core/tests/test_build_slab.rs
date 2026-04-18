use coh_core::build_slab::build_slab;
use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::types::*;
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn signature(index: u64) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-build-slab-{}", index),
        signer: "tester".to_string(),
        timestamp: 1_700_000_000 + index,
    }
}

fn defect_for_step(index: u64) -> &'static str {
    match index % 3 {
        0 => "2",
        1 => "1",
        _ => "0",
    }
}

fn create_valid_wire(index: u64, prev_digest: String, prev_state: String) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: index,
        step_type: Some("compute".to_string()),
        signatures: Some(vec![signature(index)]),
        state_hash_prev: prev_state.clone(),
        state_hash_next: format!("{:064x}", index + 1),
        chain_digest_prev: prev_digest,
        chain_digest_next: "0".repeat(64), // Must be 32 bytes hex
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "99".to_string(),
            spend: "1".to_string(),
            defect: defect_for_step(index).to_string(),
            authority: "0".to_string(),
        },
    };
    seal_wire(&mut wire);
    wire
}

fn seal_wire(wire: &mut MicroReceiptWire) {
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    wire.chain_digest_next = compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
}

#[test]
fn test_build_slab_success() {
    let w0 = create_valid_wire(0, "0".repeat(64), "0".repeat(64));
    let w1 = create_valid_wire(1, w0.chain_digest_next.clone(), w0.state_hash_next.clone());

    let res = build_slab(vec![w0, w1]);
    assert_eq!(res.decision, Decision::SlabBuilt);
    assert_eq!(res.range_start, Some(0));
    assert_eq!(res.range_end, Some(1));
}

#[test]
fn test_build_slab_reject_broken_chain() {
    let w0 = create_valid_wire(0, "0".repeat(64), "0".repeat(64));
    let mut w1 = create_valid_wire(1, w0.chain_digest_next.clone(), w0.state_hash_next.clone());
    w1.chain_digest_prev = "1".repeat(64); // Break link
    seal_wire(&mut w1);

    let res = build_slab(vec![w0, w1]);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectChainDigest));
}

#[test]
fn test_build_slab_overflow_rejected() {
    let mut w1 = create_valid_wire(0, "0".repeat(64), "0".repeat(64));
    w1.metrics.spend = (u128::MAX / 2 + 10).to_string();
    w1.metrics.v_post = "0".to_string();
    w1.metrics.v_pre = w1.metrics.spend.clone();
    seal_wire(&mut w1);

    let mut w2 = create_valid_wire(1, w1.chain_digest_next.clone(), w1.state_hash_next.clone());
    w2.metrics.spend = (u128::MAX / 2 + 10).to_string();
    w2.metrics.v_post = "0".to_string();
    w2.metrics.v_pre = w2.metrics.spend.clone();
    seal_wire(&mut w2);

    let res = build_slab(vec![w1, w2]);
    assert_eq!(res.decision, Decision::Reject);
    // Previously this was RejectOverflow at the slab level. Now the chain-level
    // cumulative drift check correctly catches it first: v_post_last + cumulative_spend
    // far exceeds v_pre_first + total_defect.
    assert!(
        res.code == Some(RejectCode::CumulativeDriftDetected)
            || res.code == Some(RejectCode::RejectOverflow),
        "Expected CumulativeDriftDetected or RejectOverflow, got {:?}",
        res.code,
    );
}
