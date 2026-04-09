#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::canon::*;
    use crate::hash::*;
    use std::convert::TryFrom;

    #[test]
    fn test_canonical_json_order() {
        let wire = MicroReceiptWire {
            schema_id: "coh.micro.v1".to_string(),
            version: 1,
            object_id: "obj_123".to_string(),
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09".to_string(),
            policy_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            step_type: "TRANSITION".to_string(),
            step_index: 0,
            chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            chain_digest_next: "0000000000000000000000000000000000000000000000000000000000000000".to_string(), // Placeholder
            state_hash_prev: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            state_hash_next: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            metrics: MetricsWire { v_pre: "100".to_string(), v_post: "80".to_string() },
            spend: "15".to_string(),
            defect: "0".to_string(),
        };

        let r = MicroReceipt::try_from(wire).unwrap();
        let prehash = to_prehash_view(&r);
        let bytes = to_canonical_json_bytes(&prehash).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();
        
        // Manual check of field order (alphabetical)
        assert!(json_str.contains("\"canon_profile_hash\""));
        // First field should be canon_profile_hash
        assert!(json_str.starts_with("{\"canon_profile_hash\""));
        
        let digest = compute_chain_digest(r.chain_digest_prev, &json_str.as_bytes());
        println!("Generated Digest: {}", digest.to_hex());
    }
}
