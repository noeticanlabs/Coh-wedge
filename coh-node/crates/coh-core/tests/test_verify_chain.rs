use coh_core::types::*;
use coh_core::verify_chain::verify_chain;
use coh_core::hash::compute_chain_digest;
use coh_core::canon::*;
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn create_valid_wire(index: u64, prev_digest: String, prev_state: String) -> MicroReceiptWire {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: index,
        state_hash_prev: prev_state.clone(),
        state_hash_next: prev_state, // Identity transition
        chain_digest_prev: prev_digest,
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "100".to_string(),
            spend: "0".to_string(),
            defect: "0".to_string(),
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
fn test_chain_accept_valid_3_steps() {
    let w0 = create_valid_wire(0, "0".repeat(64), "0".repeat(64));
    let w1 = create_valid_wire(1, w0.chain_digest_next.clone(), w0.state_hash_next.clone());
    let w2 = create_valid_wire(2, w1.chain_digest_next.clone(), w1.state_hash_next.clone());

    let res = verify_chain(vec![w0, w1, w2]);
    assert_eq!(res.decision, Decision::Accept);
    assert_eq!(res.steps_verified, 3);
    assert_eq!(res.last_step_index, 2);
}

#[test]
fn test_chain_reject_broken_digest_link() {
    let w0 = create_valid_wire(0, "0".repeat(64), "0".repeat(64));
    let mut w1 = create_valid_wire(1, w0.chain_digest_next.clone(), w0.state_hash_next.clone());
    w1.chain_digest_prev = "1".repeat(64); // Break link
    seal_wire(&mut w1); // But keep it internally valid

    let res = verify_chain(vec![w0, w1]);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectChainDigest));
    assert_eq!(res.failing_step_index, Some(1));
}
