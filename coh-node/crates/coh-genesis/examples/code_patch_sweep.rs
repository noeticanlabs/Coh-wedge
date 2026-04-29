//! Code-Patch NPE Wildness Sweep
//!
//! Runs a full benchmark sweep measuring formation admission for code patches.

use coh_genesis::code_patch::{
    build_formation_result, check_hard_gates, patch_type_for_wildness, CodePatchCandidate,
    CodePatchFirstFailure, CodePatchReport, PatchPolicy, PatchSelectorMode, RejectPathImpact,
    RejectPolicyMode,
};

// === Mulberry32 RNG ===
struct Mulberry32(u32);

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Mulberry32(seed)
    }

    fn next(&mut self) -> u32 {
        let mut t = self.0.wrapping_mul(22777);
        t = t.wrapping_add(t << 10);
        t ^= t >> 1;
        t ^= t << 15;
        t ^= t >> 17;
        self.0 = t;
        t
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u32::MAX as f64 + 1.0)
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        (self.next() as usize) % bound
    }
}

// === Patch Class Selection ===
#[derive(Clone, Debug, PartialEq)]
enum PatchClass {
    Doc,        // comments/readme only
    Test,       // adds regression tests
    Strengthen, // adds stricter reject condition
    Weaken,     // uses defaults/unwraps/removes checks
    Schema,     // changes reject enum/schema
    Refactor,   // local helper cleanup
    Envelope,   // improves semantic-envelope check
}

fn select_patch_class(wildness: f64, rand: f64) -> PatchClass {
    // At low wildness: more doc/test/refactor
    // At high wildness: more strengthen/weaken/schema/envelope
    if wildness < 0.5 {
        // Mostly doc and test
        if rand < 0.6 {
            PatchClass::Doc
        } else if rand < 0.9 {
            PatchClass::Test
        } else {
            PatchClass::Refactor
        }
    } else if wildness < 1.5 {
        // Mix of test, refactor, and some strengthen
        if rand < 0.3 {
            PatchClass::Doc
        } else if rand < 0.6 {
            PatchClass::Test
        } else if rand < 0.85 {
            PatchClass::Refactor
        } else {
            PatchClass::Strengthen
        }
    } else if wildness < 2.5 {
        // More strengthen, some weaken, some envelope
        if rand < 0.2 {
            PatchClass::Test
        } else if rand < 0.4 {
            PatchClass::Refactor
        } else if rand < 0.7 {
            PatchClass::Strengthen
        } else if rand < 0.9 {
            PatchClass::Weaken
        } else {
            PatchClass::Envelope
        }
    } else {
        // High wildness: schema changes, weaken, envelope
        if rand < 0.3 {
            PatchClass::Strengthen
        } else if rand < 0.6 {
            PatchClass::Weaken
        } else if rand < 0.8 {
            PatchClass::Schema
        } else {
            PatchClass::Envelope
        }
    }
}

fn generate_patch_text(patch_class: PatchClass, target: &str, idx: usize) -> String {
    match patch_class {
        PatchClass::Doc => format!(
            "// TODO: Document why RejectCode::SemanticEnvelopeViolation is relevant for {}",
            target
        ),
        PatchClass::Test => format!(
            "#[test]\nfn test_{}_rejects_invalid() {{\n    assert_eq!(result, Err(RejectCode::SemanticViolation));\n}}",
            target.replace(".rs", "")
        ),
        PatchClass::Strengthen => format!(
            "if declared_defect < delta_hat_{} {{\n    return Err(RejectCode::SemanticEnvelopeViolation);\n}}\n// Strict check: require positive delta",
            idx
        ),
        PatchClass::Weaken => format!(
            "let projection_hash_{} = parse_hash(input).unwrap_or_default();\n// Using default bypasses rejection",
            idx
        ),
        PatchClass::Schema => format!(
            "pub enum RejectCode {{\n    Unknown = 0,\n    SemanticEnvelopeViolation = 1,\n    NewRejectCase_{} = 2,\n}}",
            idx
        ),
        PatchClass::Refactor => format!(
            "fn helper_{}(input: &str) -> Option<String> {{\n    input.trim().to_owned()\n}}",
            idx
        ),
        PatchClass::Envelope => format!(
            "pub fn check_envelope_{}(state: &State) -> Result<Decision, RejectCode> {{\n    if state.defect >= delta_hat {{\n        return Err(RejectCode::SemanticEnvelopeViolation);\n    }}\n    Ok(Decision::Accept)\n}}",
            idx
        ),
    }
}

// === Sweep Results ===
#[derive(Clone, Debug, Default)]
struct SweepResult {
    wildness: f64,
    patch_type: &'static str,
    genesis_accept: usize,
    gate_pass: usize,
    cargo_check_pass: usize,
    cargo_test_pass: usize,
    coherence_accept: usize,
    formation_accept: usize,
    avg_novelty: f64,
    avg_genesis_margin: f64,
    avg_coherence_margin: f64,
    avg_boundary_margin: f64,
    first_failure_genesis: usize,
    first_failure_gate: usize,
    first_failure_check: usize,
    first_failure_test: usize,
    first_failure_format: usize,
    first_failure_lint: usize,
    first_failure_schema: usize,
    first_failure_coherence: usize,
    first_failure_accepted: usize,
    avg_safe_score: f64,
    max_formation_novelty: f64,
    // RejectPathImpact breakdown
    impact_none: usize,
    impact_mentioned: usize,
    impact_strengthened: usize,
    impact_weakened: usize,
    impact_schema_changed: usize,
}

fn run_sweep_one_level(
    wildness: f64,
    count: usize,
    seed: u32,
    policy: &PatchPolicy,
) -> SweepResult {
    let mut rng = Mulberry32::new(seed);
    let base_complexity = 500u128;

    let mut result = SweepResult {
        wildness,
        patch_type: patch_type_for_wildness(wildness),
        ..Default::default()
    };

    for i in 0..count {
        let target = if wildness < 2.0 {
            "semantic.rs"
        } else {
            "auth.rs"
        };
        let changed_files = if wildness < 2.0 {
            vec![target.to_string()]
        } else {
            vec![target.to_string(), "lib.rs".to_string()]
        };

        // Select patch class based on wildness to get diverse RejectPathImpact
        let patch_class = select_patch_class(wildness, rng.next_f64());
        let patch_text = generate_patch_text(patch_class, target, i);

        let changed_lines = 20 + (rng.next_f64() * wildness * 20.0) as usize;
        let generated_tokens = 100 + (rng.next_f64() * wildness * 100.0) as usize;
        let novelty = (wildness * 2.0 + rng.next_f64() * 2.0).min(10.0);

        let candidate = CodePatchCandidate {
            id: format!("sweep-{}-{}", wildness, i),
            wildness,
            target_file: target.to_string(),
            patch_text,
            changed_files,
            changed_lines,
            generated_tokens,
            novelty,
        };

        let cargo_check_fail = rng.next_f64() < (wildness / 20.0).min(0.9);
        let cargo_test_fail = rng.next_f64() < (wildness / 15.0).min(0.95);
        let format_fail = rng.next_f64() < (wildness / 25.0).min(0.8);
        let lint_fail = rng.next_f64() < (wildness / 25.0).min(0.8);
        let schema_fail = rng.next_f64() < (wildness / 10.0).min(0.95);
        let warning_rate = (wildness * 0.5) as usize;
        let test_fail_rate = if wildness >= 3.0 {
            rng.next_usize(10)
        } else {
            0
        };

        let report = CodePatchReport {
            cargo_check_pass: !cargo_check_fail,
            cargo_test_pass: !cargo_test_fail,
            fmt_pass: !format_fail,
            lint_pass: !lint_fail,
            forbidden_files_touched: false,
            schema_compat_pass: !schema_fail,
            compile_time_ms: 200 + (wildness * 100.0) as u64,
            test_time_ms: 500 + (wildness * 500.0) as u64,
            warnings: rng.next_usize(warning_rate + 1),
            failed_tests: test_fail_rate,
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
        };

        let gates = check_hard_gates(&candidate, policy);
        let gate_pass = gates.is_empty();

        let formation = build_formation_result(
            &candidate,
            base_complexity,
            &report,
            policy,
            &PatchSelectorMode::SafeNovel,
            1.0,
        );

        let genesis_accept = formation.genesis_margin >= 0;
        if genesis_accept {
            result.genesis_accept += 1;
        }
        if gate_pass {
            result.gate_pass += 1;
        }
        if report.cargo_check_pass {
            result.cargo_check_pass += 1;
        }
        if report.cargo_test_pass {
            result.cargo_test_pass += 1;
        }
        if formation.coherence_margin >= 0 {
            result.coherence_accept += 1;
        }
        if formation.formation_accept {
            result.formation_accept += 1;
            result.max_formation_novelty = result.max_formation_novelty.max(novelty);
        }

        match &formation.first_failure {
            CodePatchFirstFailure::Accepted => result.first_failure_accepted += 1,
            CodePatchFirstFailure::Genesis => result.first_failure_genesis += 1,
            CodePatchFirstFailure::CargoCheck => result.first_failure_check += 1,
            CodePatchFirstFailure::CargoTest => result.first_failure_test += 1,
            CodePatchFirstFailure::Format => result.first_failure_format += 1,
            CodePatchFirstFailure::Lint => result.first_failure_lint += 1,
            CodePatchFirstFailure::ForbiddenFile => result.first_failure_gate += 1,
            CodePatchFirstFailure::SchemaCompat => result.first_failure_schema += 1,
            CodePatchFirstFailure::Coherence => result.first_failure_coherence += 1,
        }

        // Track RejectPathImpact breakdown
        let impact = RejectPathImpact::classify(&candidate.patch_text, &candidate.changed_files);
        match impact {
            RejectPathImpact::None => result.impact_none += 1,
            RejectPathImpact::Mentioned => result.impact_mentioned += 1,
            RejectPathImpact::Strengthened => result.impact_strengthened += 1,
            RejectPathImpact::Weakened => result.impact_weakened += 1,
            RejectPathImpact::SchemaChanged => result.impact_schema_changed += 1,
        }

        result.avg_novelty += novelty;
        result.avg_genesis_margin += formation.genesis_margin as f64;
        result.avg_coherence_margin += formation.coherence_margin as f64;
        result.avg_boundary_margin += formation.boundary_margin as f64;
        result.avg_safe_score += formation.safe_score;
    }

    let n = count as f64;
    result.avg_novelty /= n;
    result.avg_genesis_margin /= n;
    result.avg_coherence_margin /= n;
    result.avg_boundary_margin /= n;
    result.avg_safe_score /= n;

    result
}

fn print_results_table(results: &[SweepResult]) {
    println!("\n=== Code-Patch Wildness Sweep Results ===");
    println!(" λ     Patch Type              Gen    Gate   Check   Test   Coh    Form");
    println!("---   -------------------- ----   ----   -----  ----  ----  ----");

    for r in results {
        println!(
            "{:5.1} {:>20} {:5} {:6} {:6} {:5} {:5} {:5}",
            r.wildness,
            r.patch_type,
            r.genesis_accept,
            r.gate_pass,
            r.cargo_check_pass,
            r.cargo_test_pass,
            r.coherence_accept,
            r.formation_accept
        );
    }
}

fn print_first_failure_table(results: &[SweepResult]) {
    println!("\n=== First Failure Distribution ===");
    println!(" λ     Genesis Gate Check Test Fmt Lint Schema Coh Accept");
    println!("---   ------ ----- ----- ---- --- ---- ------ --- ------");

    for r in results {
        println!(
            "{:5.1} {:6} {:5} {:6} {:5} {:4} {:5} {:7} {:3} {:7}",
            r.wildness,
            r.first_failure_genesis,
            r.first_failure_gate,
            r.first_failure_check,
            r.first_failure_test,
            r.first_failure_format,
            r.first_failure_lint,
            r.first_failure_schema,
            r.first_failure_coherence,
            r.first_failure_accepted
        );
    }
}

fn print_boundary_stats(results: &[SweepResult]) {
    println!("\n=== Boundary Margin Statistics ===");
    println!(" λ     AvgGenM  AvgCohM  AvgBdry AvgSafeS MaxNov");

    for r in results {
        if r.formation_accept > 0 {
            println!(
                "{:5.1} {:8.1} {:8.1} {:8.1} {:8.1} {:6.1}",
                r.wildness,
                r.avg_genesis_margin,
                r.avg_coherence_margin,
                r.avg_boundary_margin,
                r.avg_safe_score,
                r.max_formation_novelty
            );
        }
    }
}

fn main() {
    println!("Code-Patch NPE Wildness Sweep");
    println!("==========================");
    println!();

    let seed = 42;
    let count_per_level = 100;
    let levels = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0];

    println!(
        "seed: {}  count: {}  levels: {:?}",
        seed, count_per_level, levels
    );
    println!();

    // Run with Strict policy
    let mut strict_policy = PatchPolicy::default();
    strict_policy.reject_policy = RejectPolicyMode::Strict;
    println!("=== STRICT POLICY ===");
    println!(
        "Policy: forbid_unsafe={}, forbid_float={}, forbid_reject={}, mode=Strict",
        strict_policy.forbid_unsafe,
        strict_policy.forbid_float_arithmetic,
        strict_policy.forbid_reject_path_change
    );
    println!();

    let mut strict_results = Vec::new();
    for &lambda in &levels {
        let result = run_sweep_one_level(
            lambda,
            count_per_level,
            seed + lambda as u32,
            &strict_policy,
        );
        println!(
            "λ={:.1}: Gen={} Gate={} Form={}",
            lambda, result.genesis_accept, result.gate_pass, result.formation_accept
        );
        strict_results.push(result);
    }

    // Run with Audited policy
    let mut audited_policy = PatchPolicy::default();
    audited_policy.reject_policy = RejectPolicyMode::Audited;
    println!();
    println!("=== AUDITED POLICY ===");
    println!(
        "Policy: forbid_unsafe={}, forbid_float={}, forbid_reject={}, mode=Audited",
        audited_policy.forbid_unsafe,
        audited_policy.forbid_float_arithmetic,
        audited_policy.forbid_reject_path_change
    );
    println!();

    let mut audited_results = Vec::new();
    for &lambda in &levels {
        let result = run_sweep_one_level(
            lambda,
            count_per_level,
            seed + lambda as u32,
            &audited_policy,
        );
        println!(
            "λ={:.1}: Gen={} Gate={} Form={}",
            lambda, result.genesis_accept, result.gate_pass, result.formation_accept
        );
        audited_results.push(result);
    }

    // Compare results
    println!();
    print_reject_impact_table(&strict_results);

    println!();
    println!("=== COMPARISON: STRICT vs AUDITED ===");
    println!(" λ   StrictGate  AuditedGate  StrictForm  AuditedForm");
    println!("--- ---------- ----------- ---------- -----------");
    for i in 0..levels.len() {
        println!(
            "{:4.1} {:10} {:11} {:10} {:11}",
            levels[i],
            strict_results[i].gate_pass,
            audited_results[i].gate_pass,
            strict_results[i].formation_accept,
            audited_results[i].formation_accept
        );
    }

    // Summary
    println!("\n=== Summary ===");

    let (strict_lambda_star, strict_max_yield) = find_optimal(&strict_results, count_per_level);
    let (audited_lambda_star, audited_max_yield) = find_optimal(&audited_results, count_per_level);

    let strict_total = strict_results
        .iter()
        .map(|r| r.formation_accept)
        .sum::<usize>();
    let audited_total = audited_results
        .iter()
        .map(|r| r.formation_accept)
        .sum::<usize>();

    println!("");
    println!("Metric                  Strict    Audited");
    println!("------                 ------    -------");
    println!(
        "λ* (optimal):         {:6.1}   {:6.1}",
        strict_lambda_star, audited_lambda_star
    );
    println!(
        "Max yield:            {:6.2}   {:6.2}",
        strict_max_yield, audited_max_yield
    );
    println!(
        "Total accepted:       {:6}    {:6}",
        strict_total, audited_total
    );
    println!(
        "Improvement:          -        +{}",
        audited_total - strict_total
    );

    println!("\n=== Complete ===");
}

fn print_reject_impact_table(results: &[SweepResult]) {
    println!("\n=== RejectPathImpact Breakdown ===");
    println!(" λ     None  Mentioned Strengthened Weakened Schema");
    println!("---   ----  --------- ------------ --------- ------");
    for r in results {
        println!(
            "{:4.1} {:5} {:9} {:11} {:8} {:6}",
            r.wildness,
            r.impact_none,
            r.impact_mentioned,
            r.impact_strengthened,
            r.impact_weakened,
            r.impact_schema_changed
        );
    }
}

fn find_optimal(results: &[SweepResult], count_per_level: usize) -> (f64, f64) {
    let mut lambda_star = 0.0;
    let mut max_yield = 0.0;

    for r in results {
        let rate = r.formation_accept as f64 / count_per_level as f64;
        let yield_val = rate * r.avg_novelty;
        if yield_val > max_yield {
            max_yield = yield_val;
            lambda_star = r.wildness;
        }
    }

    (lambda_star, max_yield)
}
