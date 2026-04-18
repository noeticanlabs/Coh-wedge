//! Real AI workflow benchmark suite with cost analysis.

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::{build_slab, verify_chain, verify_micro};
use std::convert::TryFrom;
use std::fs;
use std::time::Instant;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() {
    println!("=== AI WORKFLOW REAL-TRACE BENCHMARK ===\n");

    let micro_json = fs::read_to_string("examples/ai_demo/ai_workflow_micro_valid.json").unwrap();
    let micro: MicroReceiptWire = serde_json::from_str(&micro_json).unwrap();

    benchmark_micro_verify_only(&micro, 10_000);
    benchmark_micro_parse_and_verify(&micro_json, 10_000);

    let sizes = [100usize, 1_000, 10_000];
    let labels = ["a", "b", "c"];

    for (i, &steps) in sizes.iter().enumerate() {
        let label = labels[i];
        let chain = generate_ai_chain(steps);
        benchmark_chain(steps, &chain, label);
        benchmark_slab(steps, chain.clone(), label);
    }

    benchmark_mixed_chain_workload(200);
    benchmark_rejection_cost(1000);

    print_memory_estimate(10_000);
}

fn benchmark_micro_verify_only(micro: &MicroReceiptWire, iterations: usize) {
    println!("[1] Real micro verify-only ({} iterations)", iterations);
    let start = Instant::now();
    let mut latencies = Vec::with_capacity(1_000);

    for i in 0..iterations {
        let t0 = Instant::now();
        let _ = verify_micro(micro.clone());
        if i < 1_000 {
            latencies.push(t0.elapsed().as_nanos() as u64);
        }
    }

    let elapsed = start.elapsed();
    let ops = iterations as f64 / elapsed.as_secs_f64();
    println!("  throughput: {:.0} ops/sec", ops);
    println!(
        "  avg latency: {:.2} µs",
        elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64
    );
    print_latency(&latencies);
    println!();
}

fn benchmark_micro_parse_and_verify(micro_json: &str, iterations: usize) {
    println!("[2] Real micro parse+verify ({} iterations)", iterations);
    let start = Instant::now();

    for _ in 0..iterations {
        let parsed: MicroReceiptWire = serde_json::from_str(micro_json).unwrap();
        let _ = verify_micro(parsed);
    }

    let elapsed = start.elapsed();
    let ops = iterations as f64 / elapsed.as_secs_f64();
    println!("  throughput: {:.0} ops/sec", ops);
    println!(
        "  avg latency: {:.2} µs",
        elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64
    );
    println!();
}

fn benchmark_chain(steps: usize, chain: &[MicroReceiptWire], label: &str) {
    println!("[3{}] Real chain verify ({} receipts)", label, steps);
    let start = Instant::now();
    let result = verify_chain(chain.to_vec());
    let elapsed = start.elapsed();
    let per_step_us = elapsed.as_secs_f64() * 1_000_000.0 / steps as f64;
    let ops = steps as f64 / elapsed.as_secs_f64();
    println!("  decision: {:?}", result.decision);
    println!("  throughput: {:.0} receipts/sec", ops);
    println!("  latency: {:.2} µs/step", per_step_us);
    println!();
}

fn benchmark_slab(steps: usize, chain: Vec<MicroReceiptWire>, label: &str) {
    println!("[4{}] Real slab build ({} receipts)", label, steps);
    let start = Instant::now();
    let result = build_slab(chain);
    let elapsed = start.elapsed();
    let per_step_us = elapsed.as_secs_f64() * 1_000_000.0 / steps as f64;
    println!("  decision: {:?}", result.decision);
    println!("  total: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  latency: {:.2} µs/receipt", per_step_us);
    println!();
}

fn benchmark_mixed_chain_workload(chains: usize) {
    println!("[5] Mixed real-trace chain workload ({} chains)", chains);
    let start = Instant::now();
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    for i in 0..chains {
        let base = generate_ai_chain(4);
        let chain = if i % 10 == 8 {
            let mut c = base.clone();
            c[2].state_hash_prev = "f".repeat(64);
            c[2].chain_digest_next = seal(&c[2]);
            c[3].chain_digest_prev = c[2].chain_digest_next.clone();
            c[3].chain_digest_next = seal(&c[3]);
            c
        } else if i % 10 == 9 {
            let mut c = base.clone();
            c[3].step_index = 4;
            c[3].chain_digest_next = seal(&c[3]);
            c
        } else {
            base
        };

        let result = verify_chain(chain);
        if result.decision == coh_core::Decision::Accept {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("  accepted: {}", accepted);
    println!("  rejected: {}", rejected);
    println!(
        "  throughput: {:.0} chains/sec",
        chains as f64 / elapsed.as_secs_f64()
    );
    println!(
        "  avg latency: {:.2} µs/chain",
        elapsed.as_secs_f64() * 1_000_000.0 / chains as f64
    );
    println!();
}

fn benchmark_rejection_cost(steps: usize) {
    println!("[6] Rejection cost analysis ({} receipts)", steps);

    // Test ACCEPT path (full chain)
    let valid_chain = generate_ai_chain(steps);
    let t0 = Instant::now();
    let _ = verify_chain(valid_chain);
    let accept_time = t0.elapsed();

    // Test REJECT path (fail at 10%)
    let mut invalid_chain = generate_ai_chain(steps);
    invalid_chain[steps / 10].step_index = 99999;
    invalid_chain[steps / 10].chain_digest_next = seal(&invalid_chain[steps / 10]);

    let t1 = Instant::now();
    let _ = verify_chain(invalid_chain);
    let reject_time = t1.elapsed();

    println!(
        "  Accept (full): {:.2} ms",
        accept_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Reject (early exit @ 10%): {:.2} ms",
        reject_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Rejection efficiency: {:.1}x faster",
        accept_time.as_secs_f64() / reject_time.as_secs_f64()
    );
    println!();
}

fn print_memory_estimate(steps: usize) {
    println!("[7] Memory profile estimate");
    let chain = generate_ai_chain(1);
    let receipt = &chain[0];

    // Rough size estimate
    let string_costs = receipt.schema_id.len()
        + receipt.version.len()
        + receipt.object_id.len()
        + receipt.canon_profile_hash.len()
        + receipt.policy_hash.len()
        + receipt.state_hash_prev.len()
        + receipt.state_hash_next.len()
        + receipt.chain_digest_prev.len()
        + receipt.chain_digest_next.len()
        + receipt.metrics.v_pre.len()
        + receipt.metrics.v_post.len()
        + receipt.metrics.spend.len()
        + receipt.metrics.defect.len()
        + receipt.metrics.authority.len();

    let struct_overhead = 256; // approximate
    let per_receipt = string_costs + struct_overhead;

    println!("  Estimated per-receipt (Wire): ~{} bytes", per_receipt);
    println!(
        "  Estimated 10k receipts: ~{:.2} MB",
        (per_receipt * steps) as f64 / 1_048_576.0
    );
    println!("  Peak allocation strategy: Linear/Reusable");
    println!();
}

fn generate_ai_chain(steps: usize) -> Vec<MicroReceiptWire> {
    let patterns = [
        ("100", "88", "12", "0"),
        ("88", "80", "7", "1"),
        ("80", "68", "11", "0"),
        ("68", "55", "12", "0"),
    ];

    let mut chain = Vec::with_capacity(steps);
    let mut prev_digest = "0".repeat(64);
    let mut prev_state = format!("{:064x}", 1u64);

    for i in 0..steps {
        let next_state = format!("{:064x}", (i + 2) as u64);
        let (v_pre, v_post, spend, defect) = patterns[i % patterns.len()];
        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("agent.workflow.demo.{}", i / 4),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: "0".repeat(64),
            step_index: i as u64,
            step_type: None,
            signatures: None,
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: defect.to_string(),
                authority: "0".to_string(),
            },
        };
        receipt.chain_digest_next = seal(&receipt);
        prev_digest = receipt.chain_digest_next.clone();
        prev_state = next_state;
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

fn print_latency(values: &[u64]) {
    let p50 = percentile(values, 50.0);
    let p95 = percentile(values, 95.0);
    let p99 = percentile(values, 99.0);
    println!(
        "  p50: {:.2} µs | p95: {:.2} µs | p99: {:.2} µs",
        p50 as f64 / 1000.0,
        p95 as f64 / 1000.0,
        p99 as f64 / 1000.0
    );
}

fn percentile(values: &[u64], p: f64) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort();
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)) as usize;
    sorted[idx.min(sorted.len() - 1)]
}
