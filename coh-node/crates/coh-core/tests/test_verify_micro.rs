use coh_core::types::*;
use coh_core::verify_micro::verify_micro;
use coh_core::hash::compute_chain_digest;
use coh_core::canon::*;
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
fn test_micro_accept() {
    let mut wire = create_valid_wire();
    seal_wire(&mut wire);
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_micro_reject_schema() {
    let mut wire = create_valid_wire();
    wire.schema_id = "wrong.id".to_string();
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectSchema));
}

#[test]
fn test_micro_reject_version() {
    let mut wire = create_valid_wire();
    wire.version = "2.0.0".to_string();
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectSchema));
}

#[test]
fn test_micro_reject_profile() {
    let mut wire = create_valid_wire();
    wire.canon_profile_hash = "1".repeat(64);
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectCanonProfile));
}

#[test]
fn test_micro_reject_empty_object() {
    let mut wire = create_valid_wire();
    wire.object_id = "".to_string();
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    // Note: Empty object_id returns RejectSchema per current verify_micro logic (as specified)
}

#[test]
fn test_micro_reject_numeric_parse() {
    let mut wire = create_valid_wire();
    wire.metrics.v_pre = "not_a_number".to_string();
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectNumericParse));
}

#[test]
fn test_micro_reject_policy_violation() {
    let mut wire = create_valid_wire();
    wire.metrics.v_post = "101".to_string(); // 101 + 20 > 100
    seal_wire(&mut wire);
    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectPolicyViolation));
}
