use crate::canon::{to_canonical_json_bytes, to_prehash_view};
use crate::hash::compute_chain_digest;
use crate::types::{MicroReceipt, MicroReceiptWire, RejectCode};
use std::convert::TryFrom;

pub fn compute_micro_digest_hex(wire: &MicroReceiptWire) -> Result<String, RejectCode> {
    let runtime = MicroReceipt::try_from(wire.clone())?;
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash)?;
    Ok(compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex())
}

pub fn finalize_micro_receipt(mut wire: MicroReceiptWire) -> Result<MicroReceiptWire, RejectCode> {
    // For FormationV2, we must ensure the projection link is valid
    if wire.profile == crate::types::AdmissionProfile::FormationV2 {
        let runtime = MicroReceipt::try_from(wire.clone())?;
        let projection = crate::verify_micro::compute_projection_hash(&runtime);
        wire.metrics.projection_hash = projection.to_hex();
    }
    
    wire.chain_digest_next = compute_micro_digest_hex(&wire)?;
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon::{
        EXPECTED_CANON_PROFILE_HASH, EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION,
    };
    use crate::types::{MetricsWire, SignatureWire};

    fn sample_wire() -> MicroReceiptWire {
        MicroReceiptWire {
            schema_id: EXPECTED_MICRO_SCHEMA_ID.to_string(),
            version: EXPECTED_MICRO_VERSION.to_string(),
            object_id: "fixture.sample".to_string(),
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: 1,
            step_type: Some("workflow".to_string()),
            signatures: Some(vec![SignatureWire {
                signature: "sig-0000000000000001".to_string(),
                signer: "fixture-signer-0".to_string(),
                timestamp: 1_700_000_001,
                authority_id: Some("fixture-signer-0".to_string()),
                scope: Some("*".to_string()),
                expires_at: None,
            }]),
            state_hash_prev: "1".repeat(64),
            state_hash_next: "2".repeat(64),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "f".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "90".to_string(),
                spend: "10".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
                ..Default::default()
            },
            profile: crate::types::AdmissionProfile::CoherenceOnlyV1,
            ..Default::default()
        }
    }

    #[test]
    fn finalized_receipt_is_deterministic() {
        let wire = sample_wire();
        let left = finalize_micro_receipt(wire.clone()).unwrap();
        let right = finalize_micro_receipt(wire).unwrap();
        assert_eq!(left.chain_digest_next, right.chain_digest_next);
    }

    #[test]
    fn metric_mutation_changes_digest() {
        let mut wire = sample_wire();
        let left = finalize_micro_receipt(wire.clone()).unwrap();
        wire.metrics.spend = "11".to_string();
        let right = finalize_micro_receipt(wire).unwrap();
        assert_ne!(left.chain_digest_next, right.chain_digest_next);
    }
}
