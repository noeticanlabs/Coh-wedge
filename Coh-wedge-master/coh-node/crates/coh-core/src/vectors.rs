#[cfg(test)]
mod tests {
    use crate::canon::*;
    use crate::hash::*;
    use crate::types::*;
    use std::convert::TryFrom;

    #[test]
    fn test_generate_step_1() {
        let wire = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "obj_123".to_string(),
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
                .to_string(),
            policy_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            step_index: 1,
            step_type: None,
            signatures: None,
            state_hash_prev: "0000000000000000000000000000000000000000000000000000000000000001"
                .to_string(),
            state_hash_next: "0000000000000000000000000000000000000000000000000000000000000002"
                .to_string(),
            chain_digest_prev: "d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149"
                .to_string(),
            chain_digest_next: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            metrics: MetricsWire {
                v_pre: "80".to_string(),
                v_post: "60".to_string(),
                spend: "20".to_string(),
                defect: "0".to_string(),
            },
        };

        let r = MicroReceipt::try_from(wire).unwrap();
        let prehash = to_prehash_view(&r);
        let bytes = to_canonical_json_bytes(&prehash).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        let digest = compute_chain_digest(r.chain_digest_prev, &json_str.as_bytes());
        println!("Generated Digest Step 1: {}", digest.to_hex());
    }
}
