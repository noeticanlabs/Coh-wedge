//! Benchmark Honest vs Hallucinated agent traces.
//! Focus: Detection efficiency at various breach points.

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::verify_chain;
use std::convert::TryFrom;
use std::time::Instant;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() {
    println!("=== COH INTEGRITY AUDIT BENCHMARK ===\n");

    let chain_size = 10_000;
    println!("Chain Size: {} receipts\n", chain_size);

    // 1. Honest Baseline
    benchmark_honest_chain(chain_size);

    // 2. Hallucinated (Early Breach - 10%)
    benchmark_hallucinated_chain(chain_size, 0.1, "Early");

    // 3. Hallucinated (Mid Breach - 50%)
    benchmark_hallucinated_chain(chain_size, 0.5, "Mid");

    // 4. Hallucinated (Late Breach - 90%)
    benchmark_hallucinated_chain(chain_size, 0.9, "Late");
}

fn benchmark_honest_chain(size: usize) {
    println!("[A] Honest Audit (100% path)");
    let chain = generate_demo_chain(size, None);

    let start = Instant::now();
    let result = verify_chain(chain);
    let elapsed = start.elapsed();

    assert_eq!(result.decision, coh_core::Decision::Accept);

    println!("  Time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!(
        "  Throughput: {:.0} receipts/sec",
        size as f64 / elapsed.as_secs_f64()
    );
    println!();
}

fn benchmark_hallucinated_chain(size: usize, breach_percent: f64, label: &str) {
    let breach_at = (size as f64 * breach_percent) as usize;
    println!(
        "[B] Hallucinated Audit ({} Breach @ step {})",
        label, breach_at
    );

    let chain = generate_demo_chain(size, Some(breach_at));

    let start = Instant::now();
    let result = verify_chain(chain);
    let elapsed = start.elapsed();

    assert_eq!(result.decision, coh_core::Decision::Reject);

    println!("  Detection Time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!(
        "  Effective Audit Speed: {:.0} steps-scanned/sec",
        breach_at as f64 / elapsed.as_secs_f64()
    );
    println!();
}

fn generate_demo_chain(steps: usize, breach_at: Option<usize>) -> Vec<MicroReceiptWire> {
    let mut chain = Vec::with_capacity(steps);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = format!("{:064x}", 1u64);

    let mut current_v = 1_000_000;

    for i in 0..steps {
        let next_state = format!("{:064x}", (i + 2) as u64);

        let v_pre = current_v;
        let spend = 10;
        let defect = 0;
        let mut v_post = v_pre - spend + defect;

        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "agent.audit.benchmark".to_string(),
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
                spend: spend.to_string(),
                defect: defect.to_string(),
            },
        };

        // If this is the breach step, we force a policy violation
        if let Some(b) = breach_at {
            if i == b {
                // v_post + spend > v_pre + defect
                // We fake a v_post that is 100 higher than it should be
                v_post += 100;
                receipt.metrics.v_post = v_post.to_string();
            }
        }

        receipt.chain_digest_next = seal(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = next_state;
        current_v = v_post;
        chain.push(receipt);
    }

    chain
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
