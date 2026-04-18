use coh_core::canon::*;
use coh_core::hash::compute_chain_digest;
use coh_core::types::*;
use std::convert::TryFrom;

#[test]
fn test_canonical_json_order() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: "0000000000000000000000000000000000000000000000000000000000000001"
            .to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
        step_index: 0,
        step_type: None,
        signatures: None,
        state_hash_prev: "0000000000000000000000000000000000000000000000000000000000000003"
            .to_string(),
        state_hash_next: "0000000000000000000000000000000000000000000000000000000000000004"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000005"
            .to_string(),
        chain_digest_next: "0000000000000000000000000000000000000000000000000000000000000006"
            .to_string(),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "15".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let r = MicroReceipt::try_from(wire).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    let json_str = String::from_utf8(bytes).unwrap();

    let expected = r#"{"canon_profile_hash":"0000000000000000000000000000000000000000000000000000000000000001","chain_digest_prev":"0000000000000000000000000000000000000000000000000000000000000005","metrics":{"authority":"0","defect":"0","spend":"15","v_post":"80","v_pre":"100"},"object_id":"demo.obj","policy_hash":"0000000000000000000000000000000000000000000000000000000000000002","schema_id":"coh.receipt.micro.v1","signatures":null,"state_hash_next":"0000000000000000000000000000000000000000000000000000000000000004","state_hash_prev":"0000000000000000000000000000000000000000000000000000000000000003","step_index":0,"step_type":null,"version":"1.0.0"}"#;

    assert_eq!(json_str, expected, "Canonical JSON drift detected!");
}

#[test]
fn test_digest_stability_vector() {
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: "0000000000000000000000000000000000000000000000000000000000000001"
            .to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
        step_index: 0,
        step_type: None,
        signatures: None,
        state_hash_prev: "0000000000000000000000000000000000000000000000000000000000000003"
            .to_string(),
        state_hash_next: "0000000000000000000000000000000000000000000000000000000000000004"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000005"
            .to_string(),
        chain_digest_next: "0".repeat(64),
        metrics: MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "15".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let r = MicroReceipt::try_from(wire).unwrap();
    let prehash = to_prehash_view(&r);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();

    let digest = compute_chain_digest(r.chain_digest_prev, &bytes);
    let hex = digest.to_hex();

    // Actual value computed by the finalized v1 logic (including authority: "0")
    let expected = "aad998c01594bc9e5041f635e029e3a36bdf7f70fff0647c814ecaa9fe59f08d";
    assert_eq!(hex, expected, "Digest semantics drift detected!");
}

#[test]
fn test_digest_self_exclusion() {
    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "demo.obj".to_string(),
        canon_profile_hash: "0".repeat(64),
        policy_hash: "0".repeat(64),
        step_index: 0,
        step_type: None,
        signatures: None,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "1".repeat(64),
        metrics: MetricsWire {
            v_pre: "10".to_string(),
            v_post: "5".to_string(),
            spend: "5".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    let r1 = MicroReceipt::try_from(wire.clone()).unwrap();
    let d1 = compute_chain_digest(
        r1.chain_digest_prev,
        &to_canonical_json_bytes(&to_prehash_view(&r1)).unwrap(),
    );

    wire.chain_digest_next = "2".repeat(64);
    let r2 = MicroReceipt::try_from(wire).unwrap();
    let d2 = compute_chain_digest(
        r2.chain_digest_prev,
        &to_canonical_json_bytes(&to_prehash_view(&r2)).unwrap(),
    );

    assert_eq!(
        d1, d2,
        "Digest recomputation must exclude chain_digest_next!"
    );
}
