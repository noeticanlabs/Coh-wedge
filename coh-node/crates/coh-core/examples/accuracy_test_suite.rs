//! Adversarial Truth Matrix Suite for Coh Validator.
//! Verifies 100% detection accuracy across multiple failure modes.

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire, RejectCode};
use coh_core::{verify_chain, Decision};
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

#[derive(Debug)]
struct TestCase {
    name: String,
    chain: Vec<MicroReceiptWire>,
    expected_decision: Decision,
    expected_reject: Option<RejectCode>,
}

fn main() {
    println!("=== COH ACCURACY & DETERMINISM SUITE ===\n");

    let mut tests = Vec::new();

    // 1. Valid Chains
    tests.push(TestCase {
        name: "Valid 10-step chain".to_string(),
        chain: generate_chain(10, None),
        expected_decision: Decision::Accept,
        expected_reject: None,
    });

    // 2. Digest Tampering
    let mut bad_digest = generate_chain(5, None);
    bad_digest[2].chain_digest_next = "f".repeat(64);
    tests.push(TestCase {
        name: "Digest Tampering (Step 2)".to_string(),
        chain: bad_digest,
        expected_decision: Decision::Reject,
        expected_reject: Some(RejectCode::RejectChainDigest),
    });

    // 3. State Discontinuity
    let mut broken_link = generate_chain(5, None);
    broken_link[3].state_hash_prev = "a".repeat(64);
    // seal it to ensure digest is valid, so failure is specifically state link
    broken_link[3].chain_digest_next = seal(&broken_link[3]);
    tests.push(TestCase {
        name: "State Discontinuity (Step 3)".to_string(),
        chain: broken_link,
        expected_decision: Decision::Reject,
        expected_reject: Some(RejectCode::RejectStateHashLink),
    });

    // 4. Policy Violation (Value Print)
    let mut breach = generate_chain(5, None);
    breach[2].metrics.v_post = "2000".to_string(); // v_pre was 1000
    breach[2].chain_digest_next = seal(&breach[2]);
    tests.push(TestCase {
        name: "Policy Violation (Value Leak)".to_string(),
        chain: breach,
        expected_decision: Decision::Reject,
        expected_reject: Some(RejectCode::RejectPolicyViolation),
    });

    // 5. Numerical Parse Error
    let mut bad_num = generate_chain(3, None);
    bad_num[1].metrics.v_pre = "not_a_number".to_string();
    // we don't seal because sealing requires valid parse
    tests.push(TestCase {
        name: "Numerical Parse Error".to_string(),
        chain: bad_num,
        expected_decision: Decision::Reject,
        expected_reject: Some(RejectCode::RejectNumericParse),
    });

    // 6. Schema Error
    let mut bad_schema = generate_chain(3, None);
    bad_schema[0].schema_id = "invalid.schema".to_string();
    tests.push(TestCase {
        name: "Schema ID Error".to_string(),
        chain: bad_schema,
        expected_decision: Decision::Reject,
        expected_reject: Some(RejectCode::RejectSchema),
    });

    // Execution
    let total = tests.len();
    let mut passed = 0;

    println!("Category               | Expected Result | Actual Result | Status");
    println!("-----------------------|-----------------|---------------|-------");

    for t in tests {
        let result = verify_chain(t.chain);
        let success = result.decision == t.expected_decision && result.code == t.expected_reject;

        if success {
            passed += 1;
        }

        println!(
            "{:<22} | {:<15?} | {:<13?} | {}",
            t.name,
            t.expected_decision,
            result.decision,
            if success { "PASS " } else { "FAIL!" }
        );

        if !success {
            println!(
                "  [ERROR] Expected {:?} / {:?}, got {:?} / {:?}",
                t.expected_decision, t.expected_reject, result.decision, result.code
            );
        }
    }

    println!(
        "\nFinal Accuracy Score: {:.2}% ({}/{})",
        (passed as f64 / total as f64) * 100.0,
        passed,
        total
    );
}

fn generate_chain(steps: usize, _breach_at: Option<usize>) -> Vec<MicroReceiptWire> {
    let mut chain = Vec::with_capacity(steps);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = format!("{:064x}", 1u64);
    let mut current_v = 1000;

    for i in 0..steps {
        let next_state = format!("{:064x}", (i + 2) as u64);
        let v_pre = current_v;
        let v_post = v_pre - 10;

        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "agent.accuracy.test".to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: None,
            signatures: None,
            state_hash_prev: prev_state.to_string(),
            state_hash_next: next_state.to_string(),
            chain_digest_prev: prev_digest.to_string(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: "10".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        };

        receipt.chain_digest_next = seal(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = next_state;
        current_v = v_post;
        chain.push(receipt);
    }
    chain
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = match MicroReceipt::try_from(receipt.clone()) {
        Ok(r) => r,
        Err(_) => return "err".to_string(), // fail if not parsable
    };
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
