//! Code-Patch Domain Adapter Demo
//!
//! Demonstrates the NPE generating patches for Coh verifier modules

use coh_genesis::code_patch::{
    build_formation_result, check_hard_gates, compute_genesis_metrics, is_formation_admissible,
    patch_type_for_wildness, CodePatchCandidate, CodePatchFirstFailure, CodePatchFormationResult,
    CodePatchReport, PatchHardGate, PatchPolicy, PatchSelectorMode,
};

fn main() {
    println!("NPE Code-Patch Domain Adapter");
    println!("==============================");
    println!();

    // Show patch types by wildness
    println!("Patch types by wildness level:");
    println!("-----------------------------");
    for lambda in [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0] {
        println!(
            "  lambda={:.1}: {}",
            lambda,
            patch_type_for_wildness(lambda)
        );
    }
    println!();

    // Default policy
    let policy = PatchPolicy::default();
    println!("Patch Policy (default):");
    println!("  allow_crypto_touch: {}", policy.allow_crypto_touch);
    println!("  allow_schema_change: {}", policy.allow_schema_change);
    println!("  max_changed_files: {}", policy.max_changed_files);
    println!("  max_changed_lines: {}", policy.max_changed_lines);
    println!("  forbid_unsafe: {}", policy.forbid_unsafe);
    println!(
        "  forbid_float_arithmetic: {}",
        policy.forbid_float_arithmetic
    );
    println!();

    // Simulate patch candidates at different wildness levels
    println!("=== First-Failure Classification Demo ===");
    println!();

    let test_cases: Vec<(f64, &str)> = vec![
        (0.0, "doc/comment/test-only patch"),
        (1.0, "semantic registry case"),
        (2.0, "envelope source/reject path"),
        (3.0, "cross-file patch"),
        (5.0, "architecture change"),
    ];

    let base_complexity = 500u128;

    for (wildness, _patch_type) in test_cases {
        let candidate = CodePatchCandidate {
            id: format!("patch-{:.*}", 1, wildness),
            wildness,
            target_file: "semantic.rs".to_string(),
            patch_text: format!("fn patch_at_wildness_{:.*}() {{}}", 1, wildness),
            changed_files: vec!["semantic.rs".to_string()],
            changed_lines: 50 + (wildness * 10.0) as usize,
            generated_tokens: (500.0 + wildness * 100.0) as usize,
            novelty: wildness * 2.0,
        };

        let (m_after, cost, defect) = compute_genesis_metrics(&candidate);
        let genesis_margin =
            base_complexity as i128 + defect as i128 - m_after as i128 - cost as i128;

        // Create a report that simulates real verification
        let report = CodePatchReport {
            cargo_check_pass: wildness < 5.0,
            cargo_test_pass: wildness < 3.0,
            fmt_pass: wildness < 4.0,
            lint_pass: wildness < 4.0,
            forbidden_files_touched: false,
            schema_compat_pass: wildness < 2.0,
            compile_time_ms: 500 + (wildness * 100.0) as u64,
            test_time_ms: 2000 + (wildness * 500.0) as u64,
            warnings: (wildness * 1.0) as usize,
            failed_tests: if wildness >= 3.0 { 1 } else { 0 },
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
        };

        let (risk, spend, def) = {
            let mut risk = 0u128;
            if !report.cargo_check_pass {
                risk += 100;
            }
            if !report.cargo_test_pass {
                risk += 80;
            }
            risk += report.warnings as u128 * 2;
            risk += report.failed_tests as u128 * 10;
            let spend =
                (report.compile_time_ms / 100) as u128 + (report.test_time_ms / 100) as u128;
            let def = 50u128;
            (risk, spend, def)
        };
        let coherence_margin = risk as i128 + def as i128 - spend as i128;

        // Classify first failure
        let first_failure =
            CodePatchFirstFailure::classify(genesis_margin, &report, coherence_margin);

        println!(
            "λ={:.1}: {} (margin: {})",
            wildness,
            first_failure.description(),
            coherence_margin
        );
    }
    println!();

    // === Hard Gates Demo ===
    println!("=== Hard Gates Demo ===");
    println!();

    let gate_test_cases = vec![
        ("patch-gates-001", "semantic.rs", "fn safe_patch() {}"),
        ("patch-gates-002", "types.rs", "fn patch_with_unsafe() {}"),
        ("patch-gates-003", "verify.rs", "fn patch_with_f64() {}"),
        ("patch-gates-004", "semantic.rs", "fn reject_check() {}"),
    ];

    for (id, target, patch_text) in gate_test_cases {
        let candidate = CodePatchCandidate {
            id: id.to_string(),
            wildness: 1.0,
            target_file: target.to_string(),
            patch_text: patch_text.to_string(),
            changed_files: if target == "semantic.rs" {
                vec!["semantic.rs".to_string(), "auth.rs".to_string()]
            } else {
                vec![target.to_string()]
            },
            changed_lines: 50,
            generated_tokens: 500,
            novelty: 5.0,
        };

        let gates = check_hard_gates(&candidate, &policy);
        if gates.is_empty() {
            println!("  {}: PASS (no hard gates)", candidate.id);
        } else {
            println!("  {}: FAIL", candidate.id);
            for gate in &gates {
                println!("    - {:?}", gate);
            }
        }
    }
    println!();

    // === Formation Result Demo ===
    println!("=== Formation Result Demo ===");
    println!();

    let candidate = CodePatchCandidate {
        id: "patch-001".to_string(),
        wildness: 2.0,
        target_file: "semantic.rs".to_string(),
        patch_text: "fn new_envelope_case() {}".to_string(),
        changed_files: vec!["semantic.rs".to_string()],
        changed_lines: 50,
        generated_tokens: 500,
        novelty: 5.0,
    };

    let report = CodePatchReport {
        cargo_check_pass: true,
        cargo_test_pass: true,
        fmt_pass: true,
        lint_pass: true,
        forbidden_files_touched: false,
        schema_compat_pass: true,
        compile_time_ms: 500,
        test_time_ms: 2000,
        warnings: 2,
        failed_tests: 0,
        genesis_margin: 0,
        coherence_margin: 0,
        formation_accept: false,
    };

    let mode = PatchSelectorMode::SafeNovel;
    let alpha = 1.0;

    let result =
        build_formation_result(&candidate, base_complexity, &report, &policy, &mode, alpha);

    println!("Candidate: {}", result.candidate_id);
    println!("  formation_accept: {}", result.formation_accept);
    println!("  first_failure: {:?}", result.first_failure);
    println!();
    println!("  Genesis margin: {}", result.genesis_margin);
    println!("  Coherence margin: {}", result.coherence_margin);
    println!("  Boundary margin: {}", result.boundary_margin);
    println!();
    println!("  Novelty: {:.1}", result.novelty);
    println!("  Safe score: {:.1}", result.safe_score);
    println!("  Edge score: {:.1}", result.edge_score);
    println!();

    if result.formation_accept {
        println!("Result: Formation-admissible safe-novel patch!");
    } else {
        let ff = result.first_failure;
        if ff == CodePatchFirstFailure::Genesis {
            println!("Result: Genesis violation - patch too complex");
        } else if ff == CodePatchFirstFailure::Coherence {
            println!("Result: Coherence violation - patch costs too much");
        } else if ff == CodePatchFirstFailure::CargoCheck {
            println!("Result: cargo check failed");
        } else if ff == CodePatchFirstFailure::CargoTest {
            println!("Result: cargo test failed");
        } else if ff == CodePatchFirstFailure::ForbiddenFile {
            println!("Result: Forbidden file touched");
        } else {
            println!("Result: Patch rejected - {:?}", ff);
        }
    }
    println!();

    // === Selector Mode Comparison ===
    println!("=== Selector Mode Comparison ===");
    println!();

    // A near-boundary candidate
    let boundary_candidate = CodePatchCandidate {
        id: "patch-boundary".to_string(),
        wildness: 2.5,
        target_file: "semantic.rs".to_string(),
        patch_text: "fn near_boundary_patch() {}".to_string(),
        changed_files: vec!["semantic.rs".to_string()],
        changed_lines: 100,
        generated_tokens: 800,
        novelty: 6.0,
    };

    // Simulate a passing report
    let boundary_report = CodePatchReport {
        cargo_check_pass: true,
        cargo_test_pass: true,
        fmt_pass: true,
        lint_pass: true,
        forbidden_files_touched: false,
        schema_compat_pass: true,
        compile_time_ms: 700,
        test_time_ms: 2500,
        warnings: 3,
        failed_tests: 0,
        genesis_margin: 0,
        coherence_margin: 0,
        formation_accept: false,
    };

    println!(
        "Candidate: {} (λ={:.1}, novelty={:.1})",
        boundary_candidate.id, boundary_candidate.wildness, boundary_candidate.novelty
    );
    println!();

    for mode in [
        PatchSelectorMode::SafeNovel,
        PatchSelectorMode::Edge,
        PatchSelectorMode::NearBoundary,
    ] {
        let result = build_formation_result(
            &boundary_candidate,
            base_complexity,
            &boundary_report,
            &policy,
            &mode,
            1.0,
        );
        println!("  Mode {:?}:", mode);
        println!("    formation_accept: {}", result.formation_accept);
        println!("    safe_score: {:.1}", result.safe_score);
        println!("    edge_score: {:.1}", result.edge_score);
        println!();
    }

    println!("=== Demo Complete ===");
}
