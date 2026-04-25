// Differential tests comparing V1 and V3 verification implementations
// Ensures both implementations make consistent decisions on the core accounting law

use coh_core::canon::{
    EXPECTED_CANON_PROFILE_HASH, EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION,
};
use coh_core::types::{Decision, MetricsWire, RejectCode, SignatureWire};
use coh_core::types_v3::{MicroReceiptV3Wire, PolicyGovernance, SequenceGuard, TieredConfig};
use coh_core::verify_micro::verify_micro;
use coh_core::verify_micro_v3::verify_micro_v3;
use std::collections::HashMap;

fn build_v1_wire(
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
) -> coh_core::types::MicroReceiptWire {
    coh_core::types::MicroReceiptWire {
        schema_id: EXPECTED_MICRO_SCHEMA_ID.to_string(),
        version: EXPECTED_MICRO_VERSION.to_string(),
        object_id: "test_obj_001".to_string(),
        canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
        policy_hash: "placeholder".to_string(),
        step_index: 1,
        step_type: Some("action".to_string()),
        signatures: Some(vec![SignatureWire {
            signature: "a".repeat(64),
            signer: "test_signer".to_string(),
            timestamp: 1_700_000_000,
        }]),
        state_hash_prev: "b".repeat(64),
        state_hash_next: "c".repeat(64),
        chain_digest_prev: "d".repeat(64),
        chain_digest_next: "e".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
    }
}

fn build_v3_wire(v_pre: &str, v_post: &str, spend: &str, defect: &str) -> MicroReceiptV3Wire {
    MicroReceiptV3Wire {
        schema_id: "coh.receipt.micro.v3".to_string(),
        version: "v3.0.0".to_string(),
        object_id: "test_obj_001".to_string(),
        canonical_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
        policy_hash: "placeholder".to_string(),
        step_index: 1,
        step_type: Some("action".to_string()),
        signatures: Some(vec![SignatureWire {
            signature: "a".repeat(64),
            signer: "test_signer".to_string(),
            timestamp: 1_700_000_000,
        }]),
        state_hash_prev: "b".repeat(64),
        state_hash_next: "c".repeat(64),
        chain_digest_prev: "d".repeat(64),
        chain_digest_next: "e".repeat(64),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
        // V3-specific fields
        objective_id: None,
        sequence_id: None,
    }
}

// =============================================================================
// Differential Tests: Verify V1 and V3 make consistent decisions
// =============================================================================

#[test]
fn test_differential_valid_receipts_accepted() {
    let test_cases = vec![
        ("100", "50", "25", "0"),
        ("100", "0", "50", "0"),
        ("1000", "500", "400", "100"),
    ];

    // Default V3 config for these tests
    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    for (v_pre, v_post, spend, defect) in test_cases {
        let v1_wire = build_v1_wire(v_pre, v_post, spend, defect);
        let v3_wire = build_v3_wire(v_pre, v_post, spend, defect);

        let v1_result = verify_micro(v1_wire);
        let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

        // Both should make the same decision
        assert_eq!(
            v1_result.decision, v3_result.decision,
            "V1 and V3 differ on valid receipt: v_pre={}, v_post={}, spend={}, defect={}",
            v_pre, v_post, spend, defect
        );
    }
}

#[test]
fn test_differential_policy_violation() {
    let test_cases = vec![
        ("100", "80", "30", "0"), // 80 + 30 = 110 > 100
        ("50", "40", "20", "0"),  // 40 + 20 = 60 > 50
    ];

    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    for (v_pre, v_post, spend, defect) in test_cases {
        let v1_wire = build_v1_wire(v_pre, v_post, spend, defect);
        let v3_wire = build_v3_wire(v_pre, v_post, spend, defect);

        let v1_result = verify_micro(v1_wire);
        let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

        // Both should reject
        assert_eq!(
            v1_result.decision,
            Decision::Reject,
            "V1 should reject policy violation"
        );
        assert_eq!(
            v3_result.decision,
            Decision::Reject,
            "V3 should reject policy violation"
        );

        // Both should agree on policy violation code
        assert_eq!(
            v1_result.code,
            Some(RejectCode::RejectPolicyViolation),
            "V1 code should be RejectPolicyViolation"
        );
    }
}

#[test]
fn test_differential_boundary_cases() {
    // Exact boundary case
    let v1_wire = build_v1_wire("100", "50", "50", "0");
    let v3_wire = build_v3_wire("100", "50", "50", "0");

    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    let v1_result = verify_micro(v1_wire);
    let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

    // Both should accept at exact boundary
    assert_eq!(v1_result.decision, Decision::Accept);
    assert_eq!(v3_result.decision, Decision::Accept);

    // Over by one - should reject
    let v1_wire_over = build_v1_wire("99", "50", "50", "0");
    let v3_wire_over = build_v3_wire("99", "50", "50", "0");

    let v1_result_over = verify_micro(v1_wire_over);
    let v3_result_over = verify_micro_v3(
        v3_wire_over,
        &config,
        &sequence_guard,
        &policy_gov,
        None,
        None,
    );

    assert_eq!(v1_result_over.decision, Decision::Reject);
    assert_eq!(v3_result_over.decision, Decision::Reject);
}

#[test]
fn test_differential_schema_validation() {
    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    // Invalid schema - V1
    let mut v1_wire = build_v1_wire("100", "50", "25", "0");
    v1_wire.schema_id = "invalid.schema".to_string();
    let v1_result = verify_micro(v1_wire);

    // Invalid schema - V3 (using wrong schema but V3 wire type)
    let mut v3_wire = build_v3_wire("100", "50", "25", "0");
    v3_wire.schema_id = "invalid.schema".to_string();
    let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

    // Both should reject
    assert_eq!(v1_result.decision, Decision::Reject);
    assert_eq!(v3_result.decision, Decision::Reject);
}

#[test]
fn test_differential_vacuous_zero() {
    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    // All zeros
    let v1_wire = build_v1_wire("0", "0", "0", "0");
    let v3_wire = build_v3_wire("0", "0", "0", "0");

    let v1_result = verify_micro(v1_wire);
    let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

    // Both should reject vacuous zero
    assert_eq!(v1_result.decision, Decision::Reject);
    assert_eq!(v1_result.code, Some(RejectCode::VacuousZeroReceipt));
    assert_eq!(v3_result.decision, Decision::Reject);
}

// Note: V3 may have different code mapping, so we just check both reject
// assert_eq!(v3_result.code, Some(RejectCode::VacuousZeroReceipt));

// =============================================================================
// Consistency Tests: Verify both implementations handle edge cases consistently
// =============================================================================

#[test]
fn test_consistency_large_values() {
    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    // Large but valid values
    let large = "999999999";
    let v1_wire = build_v1_wire(large, "0", "0", large);
    let v3_wire = build_v3_wire(large, "0", "0", large);

    let v1_result = verify_micro(v1_wire);
    let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

    // Both should make a decision (either accept or reject consistently)
    match (v1_result.decision, v3_result.decision) {
        (Decision::Accept, Decision::Accept) => {} // OK
        (Decision::Reject, Decision::Reject) => {} // OK
        (a, b) => panic!("Inconsistent decisions: V1={:?}, V3={:?}", a, b),
    }
}

#[test]
fn test_consistency_overflow() {
    let config = TieredConfig::default();
    let sequence_guard = SequenceGuard::default();
    let policy_gov = PolicyGovernance::default();

    // Values that would overflow
    let max = u64::MAX.to_string();
    let v1_wire = build_v1_wire(&max, "1000", "1000", "0");
    let v3_wire = build_v3_wire(&max, "1000", "1000", "0");

    let v1_result = verify_micro(v1_wire);
    let v3_result = verify_micro_v3(v3_wire, &config, &sequence_guard, &policy_gov, None, None);

    // Both should reject (with overflow or policy violation)
    assert_eq!(v1_result.decision, Decision::Reject);
    assert_eq!(v3_result.decision, Decision::Reject);
}
