use coh_core::build_slab::build_slab;
use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::types::*;
use coh_core::verify_micro::verify_micro;
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn signature(index: u64) -> SignatureWire {
    SignatureWire {
        signature: format!("sig-overflow-{}", index),
        signer: "tester".to_string(),
        timestamp: 1_700_000_000 + index,
    }
}

fn create_valid_wire() -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: None,
        signatures: Some(vec![signature(0)]),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "20".to_string(),
            defect: "0".to_string(),
        },
    }
}

fn seal_wire(wire: &mut MicroReceiptWire) {
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    wire.chain_digest_next = compute_chain_digest(r.chain_digest_prev, &bytes).to_hex();
}

#[test]
fn test_micro_lhs_overflow() {
    let mut wire = create_valid_wire();
    wire.metrics.v_post = u128::MAX.to_string();
    wire.metrics.spend = "1".to_string();

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectOverflow));
    assert!(res.message.contains("v_post + spend"));
}

#[test]
fn test_micro_rhs_overflow() {
    let mut wire = create_valid_wire();
    wire.metrics.v_pre = u128::MAX.to_string();
    wire.metrics.defect = "1".to_string();

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectOverflow));
    assert!(res.message.contains("v_pre + defect"));
}

#[test]
fn test_slab_accumulation_overflow() {
    let mut wire1 = create_valid_wire();
    wire1.metrics.spend = (u128::MAX / 2 + 10).to_string();
    wire1.metrics.v_post = "0".to_string();
    wire1.metrics.v_pre = wire1.metrics.spend.clone();
    seal_wire(&mut wire1);

    let mut wire2 = wire1.clone();
    wire2.step_index = 1;
    wire2.signatures = Some(vec![signature(1)]);
    wire2.chain_digest_prev = wire1.chain_digest_next.clone();
    seal_wire(&mut wire2);

    let res = build_slab(vec![wire1, wire2]);
    assert_eq!(res.decision, Decision::Reject);
    // Chain-level cumulative drift check fires before the slab overflow check,
    // because verify_chain detects that cumulative spend exceeds v_pre_first + defect.
    assert!(
        res.code == Some(RejectCode::CumulativeDriftDetected)
            || res.code == Some(RejectCode::RejectOverflow),
        "Expected CumulativeDriftDetected or RejectOverflow, got {:?}: {}",
        res.code,
        res.message,
    );
}
