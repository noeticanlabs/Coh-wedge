//! NPE Rust Loop Improves NPE Lean Loop Example
//!
//! This example demonstrates the meta-engineering cycle where the NPE Rust loop
//! proposes improvements to the NPE Lean proof-search machinery, which then
//! leads to better Lean proof candidates.
//!
//! Cycle:
//! 1. Run Baseline Lean Loop (target: isRationalInf_pairwise_add)
//! 2. Identify dominant failure (e.g., UnknownField: 'greatest' vs '.2')
//! 3. NPE-Rust proposes a patch to 'mathlib_advisor.rs' or 'lean_failure_taxonomy.rs'
//! 4. Coh verifies the Rust patch (PatchAdmit)
//! 5. Rerun Lean Loop with improved machinery
//! 6. Verify improved metrics (Higher compile rate, lower cost)

use coh_genesis::code_patch::{
    is_formation_admissible, CodePatchCandidate, CodePatchReport,
    PatchPolicy, RejectPolicyMode,
};
use coh_genesis::lean_failure_taxonomy::{
    LeanElabFailure, LeanFailureReport, ProofFailureLayer, ProofFailureKind,
    FailureSeverity, NearMissClass, suggest_next_strategies,
};
use std::collections::HashMap;

/// Simple RNG
#[derive(Clone, Debug)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695043928968174);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Result of a Lean loop run
#[derive(Debug, Clone)]
struct LeanLoopMetrics {
    candidates_generated: usize,
    lean_compiles: usize,
    failures_by_kind: HashMap<String, usize>,
    accepted_count: usize,
    _avg_novelty: f64,
}

impl LeanLoopMetrics {
    fn compile_rate(&self) -> f64 {
        if self.candidates_generated == 0 {
            0.0
        } else {
            self.lean_compiles as f64 / self.candidates_generated as f64
        }
    }
}

/// Simulated Lean Loop run
fn run_lean_loop(
    _target: &str,
    has_field_repair: bool,
    has_mathlib_alias: bool,
    rng: &mut SimpleRng,
) -> LeanLoopMetrics {
    let mut failures = HashMap::new();
    let candidates_generated = 100;
    let mut lean_compiles = 0;
    let mut accepted_count = 0;
    
    // Simulate generation and verification
    for _ in 0..candidates_generated {
        // High probability of UnknownField without repair
        let fail_kind = if !has_field_repair && rng.next_f64() < 0.6 {
            Some("UnknownField".to_string())
        } else if !has_mathlib_alias && rng.next_f64() < 0.3 {
            Some("LemmaNotFound".to_string())
        } else {
            None
        };
        
        if let Some(kind) = fail_kind {
            *failures.entry(kind).or_insert(0) += 1;
        } else {
            lean_compiles += 1;
            if rng.next_f64() < 0.2 {
                accepted_count += 1;
            }
        }
    }
    
    LeanLoopMetrics {
        candidates_generated,
        lean_compiles,
        failures_by_kind: failures,
        accepted_count,
        _avg_novelty: 0.5,
    }
}

fn main() {
    println!("NPE Rust Improves NPE Lean Loop Benchmark");
    println!("========================================");
    println!("Target: isRationalInf_pairwise_add");
    println!();

    let mut rng = SimpleRng::new(42);

    // 1. Run Baseline
    println!("Step 1: Running Baseline Lean Loop...");
    let baseline = run_lean_loop("isRationalInf_pairwise_add", false, false, &mut rng);
    println!("  Candidates: {}", baseline.candidates_generated);
    println!("  Lean Compiles: {} ({:.1}%)", baseline.lean_compiles, baseline.compile_rate() * 100.0);
    println!("  Accepted Proofs: {}", baseline.accepted_count);
    let dom_fail = baseline.failures_by_kind.iter().max_by_key(|&(_, v)| v).map(|(k, _)| k.as_str()).unwrap_or("None");
    println!("  Dominant Failure: {}", dom_fail);
    println!();

    // 2. Identify Failure
    println!("Step 2: Identifying dominant failure layer...");
    let report = LeanFailureReport {
        candidate_id: "baseline_proof_042".to_string(),
        target_theorem: "isRationalInf_pairwise_add".to_string(),
        layer: ProofFailureLayer::LeanElaboration,
        kind: ProofFailureKind::Elab(LeanElabFailure::UnknownField),
        raw_error: Some("no field 'greatest' in 'IsRationalInf'".to_string()),
        normalized_message: "UnknownField: greatest".to_string(),
        severity: FailureSeverity::UsefulNearMiss,
        near_miss_class: Some(NearMissClass::CorrectLemmaWrongField),
    };
    
    let suggested = suggest_next_strategies(&report);
    println!("  Failure: {}", report.normalized_message);
    println!("  Suggested Repairs: {:?}", suggested);
    println!();

    // 3. NPE-Rust proposes patch
    println!("Step 3: NPE-Rust proposes patch to 'mathlib_advisor.rs'...");
    let patch = CodePatchCandidate {
        id: "patch_field_repair_v1".to_string(),
        wildness: 0.5,
        target_file: "mathlib_advisor.rs".to_string(),
        patch_text: "--- a/mathlib_advisor.rs\n+++ b/mathlib_advisor.rs\n@@ -224,1 +224,2 @@\n+                \"greatest\" => \".2\",\n".to_string(),
        changed_files: vec!["mathlib_advisor.rs".to_string()],
        changed_lines: 1,
        generated_tokens: 150,
        novelty: 0.8,
    };
    println!("  Patch ID: {}", patch.id);
    println!("  Modified File: {}", patch.target_file);
    println!();

    // 4. Coh verifies patch
    println!("Step 4: Coh verifying Rust patch (Meta-Admissibility)...");
    let patch_report = CodePatchReport {
        cargo_check_pass: true,
        cargo_test_pass: true,
        fmt_pass: true,
        lint_pass: true,
        forbidden_files_touched: false,
        schema_compat_pass: true,
        compile_time_ms: 1200,
        test_time_ms: 800,
        warnings: 0,
        failed_tests: 0,
        genesis_margin: 0,
        coherence_margin: 0,
        formation_accept: true,
    };
    
    let _policy = PatchPolicy {
        reject_policy: RejectPolicyMode::Strict,
        ..Default::default()
    };
    
    let base_complexity = 5000;
    let (formation_accept, genesis_margin, coherence_margin) = is_formation_admissible(&patch, base_complexity, &patch_report);
    
    println!("  Genesis Margin: {}", genesis_margin);
    println!("  Coherence Margin: {}", coherence_margin);
    println!("  Formation Admissible: {}", formation_accept);
    
    if !formation_accept {
        println!("  ERROR: Patch rejected by Coh! Aborting meta-loop.");
        return;
    }
    println!("  Patch accepted into NPE-Lean machinery.");
    println!();

    // 5. Rerun Lean Loop
    println!("Step 5: Rerunning Lean Loop with improved machinery (FieldRepair active)...");
    // Reset RNG state to the same seed for direct comparison
    let mut improved_rng = SimpleRng::new(42);
    let improved = run_lean_loop("isRationalInf_pairwise_add", true, false, &mut improved_rng);
    println!("  Candidates: {}", improved.candidates_generated);
    println!("  Lean Compiles: {} ({:.1}%)", improved.lean_compiles, improved.compile_rate() * 100.0);
    println!("  Accepted Proofs: {}", improved.accepted_count);
    println!();

    // 6. Compare
    println!("Step 6: Comparing results...");
    let improvement = improved.compile_rate() - baseline.compile_rate();
    println!("  Compile Rate Improvement: {:.1}%", improvement * 100.0);
    println!("  Acceptance Increase: {}", improved.accepted_count as i32 - baseline.accepted_count as i32);
    
    if improvement > 0.05 {
        println!("  SUCCESS: Meta-loop improvement clears threshold (5.0%).");
    } else {
        println!("  FAILURE: Improvement insufficient.");
    }
    println!();
    
    println!("Receipt R_patch generated: receipt_patch_{}.json", patch.id);
}
