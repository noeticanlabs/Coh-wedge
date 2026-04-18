//! Execution Layer Benchmarks

use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

/// Benchmark the execution engine with different modes
fn bench_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution");

    // Create a valid receipt
    let receipt = coh_core::types::MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        step_index: 0,
        step_type: Some("workflow".to_string()),
        signatures: Some(vec![coh_core::types::SignatureWire {
            signature: "sig-0000000000000000".to_string(),
            signer: "fixture-signer-0".to_string(),
            timestamp: 1700000000,
        }]),
        state_hash_prev: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        state_hash_next: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        chain_digest_next: "431bf30f44950ef6f3d60e75bc2fd891a2f259fe218c8cf19655acf149dc85ba"
            .to_string(),
        metrics: coh_core::types::MetricsWire {
            v_pre: "100".to_string(),
            v_post: "88".to_string(),
            spend: "12".to_string(),
            defect: "0".to_string(),
        },
    };

    let action = coh_core::execute::Action {
        action_type: "dispatch_technician".to_string(),
        target: "site_alpha".to_string(),
        params: HashMap::new(),
        authority: "system".to_string(),
    };

    // Benchmark DryRun mode
    group.bench_function("dry_run", |b| {
        b.iter(|| {
            let mut engine = coh_core::execute::ExecutionEngine::new();
            engine.execute(
                receipt.clone(),
                action.clone(),
                coh_core::execute::ExecutionMode::DryRun,
            )
        });
    });

    // Benchmark Real mode
    group.bench_function("real", |b| {
        b.iter(|| {
            let mut engine = coh_core::execute::ExecutionEngine::new();
            engine.execute(
                receipt.clone(),
                action.clone(),
                coh_core::execute::ExecutionMode::Real,
            )
        });
    });

    // Benchmark Simulation mode
    group.bench_function("simulation", |b| {
        b.iter(|| {
            let mut engine = coh_core::execute::ExecutionEngine::new();
            engine.execute(
                receipt.clone(),
                action.clone(),
                coh_core::execute::ExecutionMode::Simulation,
            )
        });
    });

    group.finish();
}

/// Benchmark the verification step that happens before execution
fn bench_verify_before_execute(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_before_execute");

    let receipt = coh_core::types::MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: "agent.workflow.demo".to_string(),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        step_index: 0,
        step_type: Some("workflow".to_string()),
        signatures: Some(vec![coh_core::types::SignatureWire {
            signature: "sig-0000000000000000".to_string(),
            signer: "fixture-signer-0".to_string(),
            timestamp: 1700000000,
        }]),
        state_hash_prev: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        state_hash_next: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        chain_digest_next: "431bf30f44950ef6f3d60e75bc2fd891a2f259fe218c8cf19655acf149dc85ba"
            .to_string(),
        metrics: coh_core::types::MetricsWire {
            v_pre: "100".to_string(),
            v_post: "88".to_string(),
            spend: "12".to_string(),
            defect: "0".to_string(),
        },
    };

    group.bench_function("verify", |b| {
        b.iter(|| coh_core::verify_micro(receipt.clone()))
    });

    group.finish();
}

criterion_group!(benches, bench_execution, bench_verify_before_execute);
criterion_main!(benches);
