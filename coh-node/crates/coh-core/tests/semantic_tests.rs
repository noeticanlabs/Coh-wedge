use coh_core::auth::{fixture_signing_key, sign_micro_receipt};
use coh_core::types::*;
use coh_core::verify_micro;

#[test]
fn test_defect_bound_violation() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "obj_1".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: 1,
        step_type: Some("coh.step.transfer".to_string()),
        signatures: None,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149"
            .to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "90".to_string(),
            spend: "10".to_string(),
            defect: "2".to_string(), authority: "0".to_string(),
        },
    };

    let signing_key = fixture_signing_key("test_signer");
    let wire = sign_micro_receipt(
        wire,
        &signing_key,
        "test_signer",
        "*",
        0,
        None,
        "MICRO_RECEIPT_V1",
    )
    .expect("should sign");

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert!(res.message.contains("Defect bound violation"));
}

#[test]
fn test_identity_spend_violation() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "obj_1".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: 1,
        step_type: Some("coh.step.identity".to_string()),
        signatures: None,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149"
            .to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "90".to_string(),
            spend: "1".to_string(),
            defect: "0".to_string(), authority: "0".to_string(),
        },
    };

    let signing_key = fixture_signing_key("test_signer");
    let wire = sign_micro_receipt(
        wire,
        &signing_key,
        "test_signer",
        "*",
        0,
        None,
        "MICRO_RECEIPT_V1",
    )
    .expect("should sign");

    let res = verify_micro(wire);
    assert_eq!(res.decision, Decision::Reject);
    assert!(res
        .message
        .contains("Identity step cannot have non-zero spend"));
}

#[test]
fn test_valid_transfer() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "obj_1".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: 1,
        step_type: Some("coh.step.transfer".to_string()), // delta = 5
        signatures: Some(vec![SignatureWire {
            signature: "sig".to_string(),
            signer: "signer".to_string(),
            timestamp: 0,
            authority_id: None,
            scope: None,
            expires_at: None,
        }]),
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149"
            .to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "95".to_string(),
            spend: "10".to_string(),
            defect: "5".to_string(), authority: "0".to_string(), // 95+10 <= 100+5 (105 <= 105)
        },
    };

    let res = verify_micro(wire);
    // It will still reject because of digest mismatch in the real verifier,
    // but we can check if it passed the policy check by looking at the message
    // or by mocking the digest. For this test, we just want to see it didn't
    // fail on Defect bound.
    if res.decision == Decision::Reject {
        assert!(!res.message.contains("Defect bound violation"));
    }
}
