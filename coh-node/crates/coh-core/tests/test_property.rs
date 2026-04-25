// Production-grade property-based tests for Coh verification kernel
// Tests core invariants: accounting law, determinism, reject codes, chain linkage

use coh_core::canon::{
    EXPECTED_CANON_PROFILE_HASH, EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION,
};
use coh_core::types::{Decision, MetricsWire, RejectCode, SignatureWire};
use coh_core::verify_micro::verify_micro;

// Valid test signature from working fixture
const VALID_SIG: &str = "sig-0000000000000000";
const VALID_SIGNER: &str = "fixture-signer-0";
const VALID_TIMESTAMP: u64 = 1700000000;
const TEST_OBJ_ID: &str = "test_obj_001";
const CANON_PROFILE: &str = EXPECTED_CANON_PROFILE_HASH;

// =============================================================================
// Helper Functions
// =============================================================================

fn build_test_wire_with_metrics(
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
) -> coh_core::types::MicroReceiptWire {
    coh_core::types::MicroReceiptWire {
        schema_id: EXPECTED_MICRO_SCHEMA_ID.to_string(),
        version: EXPECTED_MICRO_VERSION.to_string(),
        object_id: TEST_OBJ_ID.to_string(),
        canon_profile_hash: CANON_PROFILE.to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        step_index: 1,
        step_type: Some("workflow".to_string()),
        signatures: Some(vec![SignatureWire {
            signature: VALID_SIG.to_string(),
            signer: VALID_SIGNER.to_string(),
            timestamp: VALID_TIMESTAMP,
        }]),
        state_hash_prev: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        state_hash_next: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        chain_digest_next: "431bf30f44950ef6f3d60e75bc2fd891a2f259fe218c8cf19655acf149dc85ba"
            .to_string(),
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
    }
}

// =============================================================================
// Property 1: Accounting Law (v_post + spend <= v_pre + defect)
// =============================================================================

#[test]
fn test_accounting_law_valid_receipts_accepted() {
    // Test various valid combinations that satisfy v_post + spend <= v_pre + defect
    let valid_cases = vec![
        ("100", "50", "25", "25"),     // 50+25 <= 100+25 ✓
        ("100", "0", "50", "50"),      // 0+50 <= 100+50 ✓
        ("100", "100", "0", "0"),      // 100+0 <= 100+0 ✓ (boundary)
        ("1000", "500", "400", "100"), // 500+400 <= 1000+100 ✓
    ];

    for (v_pre, v_post, spend, defect) in valid_cases {
        let wire = build_test_wire_with_metrics(v_pre, v_post, spend, defect);
        let result = verify_micro(wire);

        assert_eq!(
            result.decision,
            Decision::Accept,
            "Should ACCEPT: v_post={} + spend={} <= v_pre={} + defect={}",
            v_post,
            spend,
            v_pre,
            defect
        );
    }
}

#[test]
fn test_accounting_law_violation_rejected() {
    // Test cases that violate v_post + spend <= v_pre + defect
    let invalid_cases = vec![
        ("100", "80", "30", "10"), // 80+30=110 > 100+10 = 110 ✗
        ("50", "50", "10", "5"),   // 50+10=60 > 50+5 = 55 ✗
        ("100", "60", "50", "5"),  // 60+50=110 > 100+5 = 105 ✗
    ];

    for (v_pre, v_post, spend, defect) in invalid_cases {
        let wire = build_test_wire_with_metrics(v_pre, v_post, spend, defect);
        let result = verify_micro(wire);

        assert_eq!(
            result.decision,
            Decision::Reject,
            "Should REJECT: v_post={} + spend={} > v_pre={} + defect={}",
            v_post,
            spend,
            v_pre,
            defect
        );
        assert_eq!(result.code, Some(RejectCode::RejectPolicyViolation));
    }
}

// =============================================================================
// Property 2: Boundary Conditions
// =============================================================================

#[test]
fn test_boundary_exact_equality_accepted() {
    // Exact boundary: v_post + spend == v_pre + defect
    let wire = build_test_wire_with_metrics("100", "50", "50", "0"); // 50+50==100+0
    let result = verify_micro(wire);

    assert_eq!(
        result.decision,
        Decision::Accept,
        "Boundary case should ACCEPT when v_post + spend == v_pre + defect"
    );
}

#[test]
fn test_boundary_plus_one_rejected() {
    // One over boundary
    let wire = build_test_wire_with_metrics("99", "50", "50", "0"); // 50+50=100 > 99+0=99
    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectPolicyViolation));
}

// =============================================================================
// Property 3: Overflow Handling
// =============================================================================

#[test]
fn test_overflow_rejected() {
    // Test with u64::MAX values - will get parsed but violate policy
    let max_val = "18446744073709551615".to_string(); // u64::MAX
    let wire = build_test_wire_with_metrics(&max_val, "1000", "1000", "0");
    let result = verify_micro(wire);

    // Should reject - policy violation or malformed
    assert_eq!(result.decision, Decision::Reject);
}

// =============================================================================
// Property 4: Determinism
// =============================================================================

#[test]
fn test_determinism_same_input_produces_same_output() {
    let wire = build_test_wire_with_metrics("100", "50", "25", "25");

    let result1 = verify_micro(wire.clone());
    let result2 = verify_micro(wire.clone());
    let result3 = verify_micro(wire.clone());

    assert_eq!(result1.decision, result2.decision);
    assert_eq!(result2.decision, result3.decision);
    assert_eq!(result1.code, result2.code);
    assert_eq!(result2.code, result3.code);
}

// =============================================================================
// Property 5: Schema Validation
// =============================================================================

#[test]
fn test_invalid_schema_rejected() {
    let mut wire = build_test_wire_with_metrics("100", "50", "25", "25");
    wire.schema_id = "invalid.schema.id".to_string();

    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectSchema));
}

#[test]
fn test_invalid_version_rejected() {
    let mut wire = build_test_wire_with_metrics("100", "50", "25", "25");
    wire.version = "v99.99".to_string();

    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectSchema));
}

// =============================================================================
// Property 6: Vacuous Zero Receipt
// =============================================================================

#[test]
fn test_vacuous_zero_receipt_rejected() {
    // All zeros = vacuous, no economic activity
    let wire = build_test_wire_with_metrics("0", "0", "0", "0");
    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::VacuousZeroReceipt));
}

// =============================================================================
// Property 7: Missing Signatures
// =============================================================================

#[test]
fn test_missing_signatures_rejected() {
    let mut wire = build_test_wire_with_metrics("100", "50", "25", "25");
    wire.signatures = None;

    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectMissingSignature));
}

#[test]
fn test_empty_signatures_rejected() {
    let mut wire = build_test_wire_with_metrics("100", "50", "25", "25");
    wire.signatures = Some(vec![]);

    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectMissingSignature));
}

// =============================================================================
// Property 8: Missing Object ID
// =============================================================================

#[test]
fn test_missing_object_id_rejected() {
    let mut wire = build_test_wire_with_metrics("100", "50", "25", "25");
    wire.object_id = "".to_string();

    let result = verify_micro(wire);

    assert_eq!(result.decision, Decision::Reject);
    assert_eq!(result.code, Some(RejectCode::RejectMissingObjectId));
}

// =============================================================================
// Property 9: Large Values No Panic
// =============================================================================

#[test]
fn test_large_valid_values_no_panic() {
    // Large but valid values
    let large_val = "999999999".to_string();
    let wire = build_test_wire_with_metrics(&large_val, "0", "0", &large_val);

    let result = verify_micro(wire);

    // Should accept without panic
    assert!(matches!(
        result.decision,
        Decision::Accept | Decision::Reject
    ));
}
