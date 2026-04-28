#[cfg(test)]
mod tests {
    use crate::types::{Decision, MicroReceiptWire};
    use crate::verify_chain::verify_chain;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;

    fn get_adversarial_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../vectors/adversarial");
        path.push(filename);
        path
    }

    fn load_chain(path: PathBuf) -> Vec<MicroReceiptWire> {
        let file = File::open(path).expect("Failed to open adversarial vector");
        let reader = BufReader::new(file);
        reader
            .lines()
            .map(|l| serde_json::from_str(&l.unwrap()).expect("Failed to parse JSON line"))
            .collect()
    }

    #[test]
    fn test_adversarial_rejections() {
        let files = vec![
            "reject_chain_digest.jsonl",
            "reject_numeric_parse.jsonl",
            "reject_overflow.jsonl",
            "reject_policy_violation.jsonl",
            "reject_schema.jsonl",
            "reject_state_link.jsonl",
        ];

        for file in files {
            let path = get_adversarial_path(file);
            let chain = load_chain(path);
            let res = verify_chain(chain);

            assert_eq!(
                res.decision,
                Decision::Reject,
                "Adversarial file {} should have been REJECTED but was {:?}",
                file,
                res.decision
            );
            println!(
                "SUCCESS: {} rejected as expected with code {:?}",
                file, res.code
            );
        }
    }

    #[test]
    fn test_edge_cases_rejections() {
        let path = get_adversarial_path("reject_edge_cases.jsonl");
        let chain = load_chain(path);

        // Some edge cases might be per-receipt or per-chain
        // If it's a large file, we might want to check individual receipts too
        let res = verify_chain(chain);
        assert_eq!(
            res.decision,
            Decision::Reject,
            "Edge cases chain should be REJECTED"
        );
    }
}
