use coh_core::types::*;
use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::verify_micro::verify_micro;
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn create_valid_wire() -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        step_index: 0,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64), // We will compute this
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "20".to_string(),
            defect: "0".to_string(),
        },
    }
}

fn compute_digest(wire: &MicroReceiptWire) -> String {
    let r = MicroReceipt::try_from(wire.clone()).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(r.chain_digest_prev, &bytes).to_hex()
}

#[test]
fn test_tamper_metrics_changes_digest() {
    let mut wire = create_valid_wire();
    let d1 = compute_digest(&wire);

    wire.metrics.spend = "21".to_string();
    let d2 = compute_digest(&wire);

    assert_ne!(d1, d2, "Tampering with metrics must change the digest!");
}

#[test]
fn test_tamper_prev_link_changes_digest() {
    let mut wire = create_valid_wire();
    let d1 = compute_digest(&wire);

    wire.chain_digest_prev = "1".repeat(64);
    let d2 = compute_digest(&wire);

    assert_ne!(d1, d2, "Tampering with chain_digest_prev must change the digest!");
}

#[test]
fn test_tamper_state_link_changes_digest() {
    let mut wire = create_valid_wire();
    let d1 = compute_digest(&wire);

    wire.state_hash_next = "1".repeat(64);
    let d2 = compute_digest(&wire);

    assert_ne!(d1, d2, "Tampering with state_hash_next must change the digest!");
}

#[test]
fn test_verifier_traps_tampered_receipt() {
    let mut wire = create_valid_wire();
    let correct_digest = compute_digest(&wire);
    wire.chain_digest_next = correct_digest;

    // 1. Verify it accepts initially
    let res = verify_micro(wire.clone());
    assert_eq!(res.decision, Decision::Accept, "Initial verify should accept but failed: {}", res.message);

    // 2. Tamper with a field in a way that stays POLICY-VALID but changes the digest
    wire.metrics.spend = "10".to_string(); // Change spend (valid policy: 80+10 <= 100)
    
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectChainDigest), "Verifier should have detected the digest mismatch! Got: {:?}", res.code);
}
