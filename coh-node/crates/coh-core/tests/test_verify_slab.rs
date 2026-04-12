use coh_core::types::*;
use coh_core::verify_slab_envelope;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn create_valid_slab() -> SlabReceiptWire {
    SlabReceiptWire {
        schema_id: "coh.receipt.slab.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: VALID_PROFILE.to_string(),
        policy_hash: "0".repeat(64),
        range_start: 0,
        range_end: 1,
        micro_count: 2,
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        state_hash_first: "0".repeat(64),
        state_hash_last: "0".repeat(64),
        merkle_root: "0".repeat(64),
        summary: SlabSummaryWire {
            total_spend: "20".to_string(),
            total_defect: "0".to_string(),
            v_pre_first: "100".to_string(),
            v_post_last: "80".to_string(),
        },
    }
}

#[test]
fn test_verify_slab_envelope_accept() {
    let wire = create_valid_slab();
    let res = verify_slab_envelope(wire);
    assert_eq!(res.decision, Decision::Accept);
}

#[test]
fn test_verify_slab_envelope_reject_policy() {
    let mut wire = create_valid_slab();
    wire.summary.v_post_last = "150".to_string(); // 150 + 20 > 100
    let res = verify_slab_envelope(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectPolicyViolation));
}

#[test]
fn test_verify_slab_envelope_reject_count() {
    let mut wire = create_valid_slab();
    wire.micro_count = 0;
    let res = verify_slab_envelope(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectSlabSummary));
}

#[test]
fn test_verify_slab_envelope_reject_range() {
    let mut wire = create_valid_slab();
    wire.range_start = 10;
    wire.range_end = 5;
    let res = verify_slab_envelope(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert_eq!(res.code, Some(RejectCode::RejectSlabSummary));
}
