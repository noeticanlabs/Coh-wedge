#![allow(clippy::needless_update)]
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
            authority_id: Some("fixture-signer-0".to_string()),
            scope: Some("*".to_string()),
            expires_at: None,
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
            authority: "0".to_string(),
            ..Default::default()
        },
        profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
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
            authority_id: Some("fixture-signer-0".to_string()),
            scope: Some("*".to_string()),
            expires_at: None,
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
            authority: "0".to_string(),
            ..Default::default()
        },
        profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
        ..Default::default()
    };

    group.bench_function("verify", |b| {
        b.iter(|| coh_core::verify_micro(receipt.clone()))
    });

    group.finish();
}

/// Benchmark the Coh Physics Hierarchy (Bit -> Atom -> Spinor -> Yang-Mills)
fn bench_physics_hierarchy(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_hierarchy");
    
    use coh_core::cohbit::{CohBit, CohBitState};
    use coh_core::atom::{CohAtom, AtomGeometry, AtomMetabolism};
    use coh_physics::CohSpinor;
    use coh_physics::current::CoherenceCurrent;
    use coh_core::types::{Hash32, Decision};
    use num_rational::Rational64;
    use num_complex::Complex64;

    let state_x = Hash32([0; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([1; 32]),
        transition_id: "bench".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(2, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
        ..Default::default()
    };

    // 1. Bit Admissibility (1,000 operations)
    group.bench_function("bit_admissibility_x1000", |b| {
        let bit = criterion::black_box(bit);
        b.iter(|| {
            for _ in 0..1000 {
                let _ = criterion::black_box(bit.margin());
                let _ = criterion::black_box(bit.is_executable());
            }
        });
    });

    // 2. Atom Optimal Selection (1,000 candidates)
    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone(); 1000],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.5,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(5, 1),
        },
        receipt_chain: vec![],
    };
    group.bench_function("atom_selection_1000_candidates", |b| {
        let atom = criterion::black_box(atom);
        b.iter(|| {
            let _ = criterion::black_box(atom.select_optimal_bit(1.0, 5.0));
        });
    });

    // 3. Spinor Current Computation (1,000 operations)
    let psi = CohSpinor::new(
        Complex64::new(1.0, 0.0),
        Complex64::new(0.5, 0.2),
        Complex64::new(0.0, -0.1),
        Complex64::new(0.1, 0.0),
    );
    group.bench_function("spinor_current_x1000", |b| {
        let psi = criterion::black_box(psi);
        b.iter(|| {
            for _ in 0..1000 {
                let _ = criterion::black_box(CoherenceCurrent::compute(&psi));
            }
        });
    });

    // 4. Effective Metric Coupling (1,000 operations)
    let current = CoherenceCurrent::compute(&psi);
    let g_base = [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, -1.0]];
    group.bench_function("metric_coupling_x1000", |b| {
        let current = criterion::black_box(current);
        let g_base = criterion::black_box(g_base);
        b.iter(|| {
            for _ in 0..1000 {
                let _ = criterion::black_box(current.effective_metric_coupling(g_base, 0.1, 0.05, 0.02));
            }
        });
    });

    // 5. Yang-Mills Curvature (1,000 operations)
    use coh_physics::gauge::CohGaugeField;
    let mut gauge = CohGaugeField::new(3);
    gauge.connection[0][0] = 0.1;
    gauge.connection[1][1] = 0.1;
    group.bench_function("yang_mills_curvature_x1000", |b| {
        let gauge = criterion::black_box(gauge.clone());
        b.iter(|| {
            for _ in 0..1000 {
                let _ = criterion::black_box(gauge.compute_curvature(0, 1));
            }
        });
    });

    // 6. Wilson Loop Holonomy (100 steps, 100 iterations)
    use coh_physics::gauge::WilsonLoopReceipt;
    use coh_core::trajectory::path_integral::CohHistory;
    let history = CohHistory {
        steps: vec![CohBit {
            from_state: state_x,
            to_state: Hash32([1; 32]),
            rv_status: Decision::Accept,
            ..Default::default()
        }; 100],
    };
    group.bench_function("wilson_loop_100_steps_x100", |b| {
        let history = criterion::black_box(history.clone());
        let gauge = criterion::black_box(gauge.clone());
        b.iter(|| {
            for _ in 0..100 {
                let _ = criterion::black_box(WilsonLoopReceipt::compute_holonomy(&history, &gauge));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_execution, bench_verify_before_execute, bench_physics_hierarchy);
criterion_main!(benches);
