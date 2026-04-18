#[cfg(test)]
mod tests {
    use crate::measurement::{map_chain, verify_chain_dissipation, Measurement};
    use crate::types::{Hash32, Metrics, MicroReceipt};
    use std::convert::TryInto;

    struct Identity;
    impl Measurement for Identity {
        fn map_step(
            &self,
            pre: &Hash32,
            receipt: &MicroReceipt,
            post: &Hash32,
        ) -> Option<(Hash32, MicroReceipt, Hash32)> {
            Some((*pre, receipt.clone(), *post))
        }
    }

    struct Compression;
    impl Measurement for Compression {
        fn map_step(
            &self,
            pre: &Hash32,
            receipt: &MicroReceipt,
            post: &Hash32,
        ) -> Option<(Hash32, MicroReceipt, Hash32)> {
            // Collapse all states to zero hash
            let zero = Hash32([0; 32]);
            let mut r = receipt.clone();
            r.state_hash_prev = zero;
            r.state_hash_next = zero;
            Some((zero, r, zero))
        }
    }

    /// Reflection Violation: A measurement that "invents validity".
    /// It maps an "illegal" source receipt to a "legal" target receipt.
    struct ReflectionViolator;
    impl Measurement for ReflectionViolator {
        fn map_step(
            &self,
            _pre: &Hash32,
            receipt: &MicroReceipt,
            _post: &Hash32,
        ) -> Option<(Hash32, MicroReceipt, Hash32)> {
            // Scenario: Source receipt has a defect (negative potential drift).
            // This violator "cleans" it by setting defect to zero.
            let mut r = receipt.clone();
            r.metrics.defect = 0;
            Some((r.state_hash_prev, r.clone(), r.state_hash_next))
        }
    }

    fn mock_receipt(v_pre: u128, v_post: u128, spend: u128, defect: u128) -> MicroReceipt {
        use crate::types::Metrics;
        MicroReceipt {
            schema_id: "test".to_string(),
            version: "1.0.0".to_string(),
            object_id: "obj".to_string(),
            canon_profile_hash: Hash32([0; 32]),
            policy_hash: Hash32([0; 32]),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: Hash32([1; 32]),
            state_hash_next: Hash32([2; 32]),
            chain_digest_prev: Hash32([0; 32]),
            chain_digest_next: Hash32([0; 32]),
            metrics: Metrics {
                v_pre,
                v_post,
                spend,
                defect,
                authority: 0,
            },
        }
    }

    #[test]
    fn test_identity_oplax() {
        let chain = vec![mock_receipt(100, 80, 20, 0)];
        assert!(verify_chain_dissipation(&Identity, &chain));
    }

    #[test]
    fn test_dissipation_violation() {
        struct ExpensiveMeasurement;
        impl Measurement for ExpensiveMeasurement {
            fn map_step(
                &self,
                p: &Hash32,
                r: &MicroReceipt,
                n: &Hash32,
            ) -> Option<(Hash32, MicroReceipt, Hash32)> {
                let mut mapped = r.clone();
                mapped.metrics.spend += 10; // Increase cost!
                Some((*p, mapped, *n))
            }
        }

        let chain = vec![mock_receipt(100, 80, 20, 0)];
        // Should fail oplax dissipation check
        assert!(!verify_chain_dissipation(&ExpensiveMeasurement, &chain));
    }

    #[test]
    fn test_collapse_detection() {
        use crate::measurement::detect_collapse;
        let chain1 = vec![mock_receipt(100, 80, 20, 0)]; // state 1 -> 2
        let mut chain2 = vec![mock_receipt(100, 80, 20, 0)];
        chain2[0].state_hash_prev = Hash32([3; 32]); // state 3 -> 2

        let traces = vec![chain1, chain2];
        let collapses = detect_collapse(&Compression, &traces);

        // Compression maps s1 and s3 to t0.
        assert!(collapses.len() > 0);
        assert!(collapses[0].source_hashes.contains(&Hash32([1; 32])));
        assert!(collapses[0].source_hashes.contains(&Hash32([3; 32])));
    }
}
