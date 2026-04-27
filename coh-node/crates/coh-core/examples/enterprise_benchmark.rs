use coh_core::types::{Decision, SlabReceiptWire, SlabSummaryWire};
use coh_core::verify_slab::verify_slab_envelope;
use std::time::Instant;

fn main() {
    println!("=== Enterprise Benchmark Starting ===");

    // Create a realistic-ish slab for benchmarking
    let slab = SlabReceiptWire {
        schema_id: "coh.receipt.slab.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "bench-obj-001".to_string(),
        canon_profile_hash: "00".repeat(32),
        policy_hash: "00".repeat(32),
        range_start: 0,
        range_end: 999,
        micro_count: 1000,
        chain_digest_prev: "00".repeat(32),
        chain_digest_next: "00".repeat(32),
        state_hash_first: "00".repeat(32),
        state_hash_last: "00".repeat(32),
        merkle_root: "00".repeat(32),
        summary: SlabSummaryWire {
            total_spend: "1000000".to_string(),
            total_defect: "10000".to_string(), authority: "0".to_string(),
            v_pre_first: "5000000".to_string(),
            v_post_last: "4010000".to_string(),
        },
    };

    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let result = verify_slab_envelope(slab.clone());
        assert_eq!(result.decision, Decision::Accept);
    }

    let duration = start.elapsed();
    let per_iter = duration.as_secs_f64() / iterations as f64;
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!("Completed {} iterations", iterations);
    println!("Total time: {:?}", duration);
    println!("Throughput: {:.2} verifications/sec", throughput);
    println!("Latency: {:.2} ns/verification", per_iter * 1_000_000_000.0);
    println!("=== Enterprise Benchmark Finished ===");
}
