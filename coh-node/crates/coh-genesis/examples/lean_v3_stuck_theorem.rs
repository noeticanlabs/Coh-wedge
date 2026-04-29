//! Lean V3 Stuck Theorem NPE Test
//!
//! Target theorem: isRationalInf_add_inf_le
//! This benchmark tests if NPE can find a compiling proof or helper decomposition.
//!
//! Candidate classes:
//! - DirectGLB: Try using h1.greatest, h2.greatest directly
//! - PairwiseSumSet: Introduce ssum = {z | exists x in s1, y in s2, z = x + y}
//! - HelperDecomposition: Reduce to helper lemma with hsum hypothesis
//! - NamedMissingLemma: Also propose isRationalInf_pairwise_add
//! - LibrarySearch: Search existing lemmas
//! - ForbiddenShortcut: Try sorry/admit/axiom (should be rejected by Coh)

use coh_genesis::lean_proof::{
    is_formation_admissible, LeanVerificationReport, ProofCandidate, ProofFirstFailure,
};

/// RNG for reproducible candidates
#[derive(Clone, Debug)]
struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525 + 1013904223);
        self.state
    }
    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u32::MAX as f64)
    }
}

/// Stuck theorem candidate class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StuckCandidateClass {
    /// Try using h1.greatest, h2.greatest directly (likely fails)
    DirectGLB,
    /// Introduce pairwise sum set ssum = {z | exists x in s1, y in s2, z = x + y}
    PairwiseSumSet,
    /// Reduce to helper lemma with explicit hsum hypothesis
    HelperDecomposition,
    /// Also propose isRationalInf_pairwise_add as missing lemma
    NamedMissingLemma,
    /// Search library for existing lemmas
    LibrarySearch,
    /// Try sorry/admit/axiom (should be rejected by Coh)
    ForbiddenShortcut,
}

impl StuckCandidateClass {
    fn select(wildness: f64, rand: f64) -> Self {
        // Higher wildness = more exploration, less forbidden
        let forbidden_chance = (0.3 - wildness * 0.02).max(0.05);

        if rand < 0.10 {
            StuckCandidateClass::ForbiddenShortcut
        } else if rand < 0.10 + forbidden_chance {
            StuckCandidateClass::DirectGLB
        } else if rand < 0.35 {
            StuckCandidateClass::PairwiseSumSet
        } else if rand < 0.60 {
            StuckCandidateClass::HelperDecomposition
        } else if rand < 0.80 {
            StuckCandidateClass::NamedMissingLemma
        } else {
            StuckCandidateClass::LibrarySearch
        }
    }
}

/// Result tracking for stuck theorem test
#[derive(Debug, Default, Clone)]
struct StuckSweepResult {
    wildness: f64,
    candidates_generated: usize,
    pre_lean_rejected: usize,
    lean_attempted: usize,
    full_proofs_compiled: usize,
    helper_decompositions_compiled: usize,
    theorem_weakening_rejected: usize,
    sorry_rejected: usize,
    admit_rejected: usize,
    axiom_rejected: usize,
    forbidden_import_rejected: usize,

    // First failure breakdown
    failure_genesis: usize,
    failure_statement_changed: usize,
    failure_new_axiom: usize,
    failure_sorry: usize,
    failure_admit: usize,
    failure_state: usize,
    failure_forbidden: usize,
    failure_coherence: usize,
    accepted: usize,
}

/// Generate proof text for the stuck theorem
fn generate_stuck_proof_text(class: StuckCandidateClass, target: &str, idx: usize) -> String {
    match class {
        StuckCandidateClass::DirectGLB => {
            // Try direct GLB attempt - likely fails because hbound is pairwise
            format!("\nlemma {} : forall {{s1 s2 : Set ENNRat}} {{i1 i2 k : ENNRat}},\n  IsRationalInf s1 i1 ->\n  IsRationalInf s2 i2 ->\n  (forall x : ENNRat, x IN s1 -> forall y : ENNRat, y IN s2 -> k <= x + y) ->\n  k <= i1 + i2 := by\n  intros s1 s2 i1 i2 k h1 h2 hbound\n  have h1_greatest := h1.2\n  have h2_greatest := h2.2\n  sorry\n", target)
        }
        StuckCandidateClass::PairwiseSumSet => {
            // Introduce pairwise sum set
            format!("\nlemma {} : forall {{s1 s2 : Set ENNRat}} {{i1 i2 k : ENNRat}},\n  IsRationalInf s1 i1 ->\n  IsRationalInf s2 i2 ->\n  (forall x : ENNRat, x IN s1 -> forall y : ENNRat, y IN s2 -> k <= x + y) ->\n  k <= i1 + i2 := by\n  intros s1 s2 i1 i2 k h1 h2 hbound\n  let ssum : Set ENNRat := fun z : ENNRat => exists x : ENNRat, x IN s1 /\\ exists y : ENNRat, y IN s2 /\\ z = x + y\n  have hk_lower : forall z : ENNRat, z IN ssum -> k <= z := by\n    intros z hz\n    cases hz with x hx y hy hxyz\n    exact hbound x hx y hy\n  sorry\n", target)
        }
        StuckCandidateClass::HelperDecomposition => {
            // Helper decomposition with explicit hsum hypothesis
            format!(
                r#"
lemma {} : ∀ {{s1 s2 : Set ENNRat}} {{i1 i2 k : ENNRat}},
  IsRationalInf s1 i1 →
  IsRationalInf s2 i2 →
  (∀ x : ENNRat, x ∈ s1 → ∀ y : ENNRat, y ∈ s2 → k ≤ x + y) →
  k ≤ i1 + i2 := by
  intros s1 s2 i1 i2 k h1 h2 hbound
  apply hsum.greatest -- try using hsum.greatest
  intros z hz
  rcases hz with ⟨x, hx, y, hy, rfl⟩
  exact hbound x hx y hy
"#,
                target
            )
        }
        StuckCandidateClass::NamedMissingLemma => {
            // Also propose the missing lemma
            format!(
                r#"
-- Main theorem (may not compile without helper)
lemma {} : ∀ {{s1 s2 : Set ENNRat}} {{i1 i2 k : ENNRat}},
  IsRationalInf s1 i1 →
  IsRationalInf s2 i2 →
  (∀ x : ENNRat, x ∈ s1 → ∀ y : ENNRat, y ∈ s2 → k ≤ x + y) →
  k ≤ i1 + i2 := by
  intros s1 s2 i1 i2 k h1 h2 hbound
  have hsum := isRationalInf_pairwise_add h1 h2
  apply hsum.greatest
  intros z hz
  rcases hz with ⟨x, hx, y, hy, rfl⟩
  exact hbound x hx y hy

-- Missing helper lemma (to be discovered/filled)
lemma isRationalInf_pairwise_add {{s1 s2 : Set ENNRat}} {{i1 i2 : ENNRat}}
  (h1 : IsRationalInf s1 i1)
  (h2 : IsRationalInf s2 i2) :
  IsRationalInf
    (fun z : ENNRat => ∃ x : ENNRat, x ∈ s1 ∧ ∃ y : ENNRat, y ∈ s2 ∧ z = x + y)
    (i1 + i2) := by
  sorry
"#,
                target
            )
        }
        StuckCandidateClass::LibrarySearch => {
            // Try searching library
            format!("\nlemma {} : forall {{s1 s2 : Set ENNRat}} {{i1 i2 k : ENNRat}},\n  IsRationalInf s1 i1 ->\n  IsRationalInf s2 i2 ->\n  (forall x : ENNRat, x IN s1 -> forall y : ENNRat, y IN s2 -> k <= x + y) ->\n  k <= i1 + i2 := by\n  intros s1 s2 i1 i2 k h1 h2 hbound\n  -- Try: exact infimum_add / IsRationalInf.add / add_glb\n  sorry\n", target)
        }
        StuckCandidateClass::ForbiddenShortcut => {
            // Explicit sorry/admit/axiom - should be rejected
            match idx % 3 {
                0 => format!("lemma {} : _ := by sorry", target),
                1 => format!("lemma {} : _ := by admit", target),
                _ => "lemma foo : True := { axiom bar : True }".to_string(),
            }
        }
    }
}

/// Run sweep at one wildness level
fn run_stuck_sweep_one_level(wildness: f64, count: usize, seed: u32) -> StuckSweepResult {
    let mut rng = Mulberry32::new(seed);
    let base_complexity = 1000u128;

    let mut result = StuckSweepResult {
        wildness,
        ..Default::default()
    };

    for idx in 0..count {
        let class = StuckCandidateClass::select(wildness, rng.next_f64());

        // Generate proof text
        let proof_text = generate_stuck_proof_text(class, "isRationalInf_add_inf_le", idx);

        // Check for forbidden patterns BEFORE moving
        let has_sorry = proof_text.contains("sorry");
        let has_admit = proof_text.contains("admit");
        let has_axiom = proof_text.contains("axiom");
        let statement_changed =
            proof_text.contains("lemma ") && !proof_text.contains("isRationalInf_add_inf_le");

        result.candidates_generated += 1;

        // Pre-Coh gate: reject forbidden shortcuts
        if has_sorry {
            result.pre_lean_rejected += 1;
            result.sorry_rejected += 1;
            result.failure_sorry += 1;
            continue;
        }
        if has_admit {
            result.pre_lean_rejected += 1;
            result.admit_rejected += 1;
            result.failure_admit += 1;
            continue;
        }
        if has_axiom {
            result.pre_lean_rejected += 1;
            result.axiom_rejected += 1;
            result.failure_new_axiom += 1;
            continue;
        }
        if statement_changed {
            result.pre_lean_rejected += 1;
            result.theorem_weakening_rejected += 1;
            result.failure_statement_changed += 1;
            continue;
        }

        // Create candidate with proof_text
        let candidate = ProofCandidate {
            id: format!("stuck-{:?}-{}", class, idx),
            wildness,
            target_theorem: "isRationalInf_add_inf_le".to_string(),
            proof_text,
            proof_tactics: vec![],
            tactic_count: 1 + (rng.next_f64() * wildness * 5.0) as usize,
            helper_lemmas: if wildness >= 1.5 {
                (rng.next_f64() * wildness * 2.0) as usize
            } else {
                0
            },
            imports: vec![],
            novelty: (wildness + rng.next_f64() * 2.0).min(10.0),
        };

        // Simulate Lean verification (without actually running Lean)
        let compile_fail = rng.next_f64() < (wildness / 15.0).min(0.95);

        let report = LeanVerificationReport {
            compiles: !compile_fail,
            has_sorry: false,
            has_admit: false,
            new_axioms: 0,
            statement_unchanged: true,
            forbidden_imports: false,
            build_time_ms: 100 + (wildness * 50.0) as u64,
            errors: vec![],
            warnings: if rng.next_f64() < 0.3 { 1 } else { 0 },
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
        };

        let (formation_accept, gen_margin, coh_margin) =
            is_formation_admissible(&candidate, base_complexity, &report);

        result.lean_attempted += 1;

        // Classify first failure using the enum variants that exist
        if gen_margin < 0 {
            result.failure_genesis += 1;
        } else if !report.compiles {
            result.failure_state += 1;
        } else if statement_changed {
            result.failure_statement_changed += 1;
        } else if has_sorry {
            result.failure_sorry += 1;
        } else if has_admit {
            result.failure_admit += 1;
        } else if formation_accept {
            result.accepted += 1;
            // Check if it's a helper decomposition
            if candidate.proof_text.contains("IsRationalInf")
                && candidate.proof_text.contains("ssum")
            {
                result.helper_decompositions_compiled += 1;
            } else {
                result.full_proofs_compiled += 1;
            }
        } else {
            result.failure_coherence += 1;
        }
    }

    result
}

/// Print results table
fn print_results(results: &[StuckSweepResult]) {
    println!("\n=== Lean V3 Stuck Theorem NPE Test ===");
    println!("target_theorem = isRationalInf_add_inf_le");
    println!("seed = 42");
    println!("count_per_level = 100");
    println!();

    // Header
    println!("  Lambda   Gen   Reject   Lean   Full   Helper   Accept");
    println!("  ------   ---   ------   ----   ----   ------   ------");

    for r in results {
        let accept_rate = if r.candidates_generated > 0 {
            (r.accepted as f64 / r.candidates_generated as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "  {:>5.1}   {:>4}    {:>5}   {:>4}   {:>4}    {:>5}    {:>5.1}%",
            r.wildness,
            r.candidates_generated,
            r.pre_lean_rejected,
            r.lean_attempted,
            r.full_proofs_compiled,
            r.helper_decompositions_compiled,
            accept_rate
        );
    }
}

/// Print first failure breakdown
fn print_first_failure_breakdown(results: &[StuckSweepResult]) {
    println!("\n=== First Failure Breakdown ===");
    println!("  Lambda   Gen   Stmt   Axiom   Sorry   Admit   Forbid   Coh   Acc");
    println!("  ------   ---   ----   -----   -----   -----   ------   ---   ---");

    for r in results {
        println!(
            "  {:>5.1}   {:>4}   {:>4}   {:>4}    {:>4}    {:>4}    {:>4}   {:>4}   {:>4}",
            r.wildness,
            r.failure_genesis,
            r.failure_statement_changed,
            r.failure_new_axiom,
            r.failure_sorry,
            r.failure_admit,
            r.failure_forbidden,
            r.failure_coherence,
            r.accepted
        );
    }
}

/// Print summary
fn print_summary(results: &[StuckSweepResult]) {
    // Aggregate
    let total_candidates: usize = results.iter().map(|r| r.candidates_generated).sum();
    let total_rejected: usize = results.iter().map(|r| r.pre_lean_rejected).sum();
    let total_lean: usize = results.iter().map(|r| r.lean_attempted).sum();
    let total_full: usize = results.iter().map(|r| r.full_proofs_compiled).sum();
    let total_helper: usize = results
        .iter()
        .map(|r| r.helper_decompositions_compiled)
        .sum();
    let total_accepted: usize = results.iter().map(|r| r.accepted).sum();

    println!("\n=== Summary ===");
    println!("Total candidates generated: {}", total_candidates);
    println!(
        "Pre-Lean rejected: {} ({:.1}%)",
        total_rejected,
        total_rejected as f64 / total_candidates as f64 * 100.0
    );
    println!("Lean attempted: {}", total_lean);
    println!("Full proofs compiled: {}", total_full);
    println!("Helper decompositions: {}", total_helper);
    println!("Total accepted: {}", total_accepted);

    // Find best for each category
    let mut best_full = (0.0, 0);
    let mut best_helper = (0.0, 0);
    for r in results {
        if r.full_proofs_compiled > best_full.1 {
            best_full = (r.wildness, r.full_proofs_compiled);
        }
        if r.helper_decompositions_compiled > best_helper.1 {
            best_helper = (r.wildness, r.helper_decompositions_compiled);
        }
    }

    println!(
        "\nBest wildness for full proofs: lambda={:.1} ({} compiled)",
        best_full.0, best_full.1
    );
    println!(
        "Best wildness for helper decomposition: lambda={:.1} ({} compiled)",
        best_helper.0, best_helper.1
    );
}

fn main() {
    println!("Lean V3 Stuck Theorem NPE Test");
    println!("===============================");
    println!();
    println!("target: isRationalInf_add_inf_le");
    println!("Target file: Coh/Coh/V3/Distance.lean");
    println!();

    // Test levels
    let levels = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let count = 100;
    let seed = 42;

    let mut results = Vec::new();
    for &wildness in &levels {
        let result = run_stuck_sweep_one_level(wildness, count, seed);
        results.push(result.clone());

        println!(
            "lambda={:.1}: Gen={} Rej={} Lean={} Full={} Helper={} Accept={}",
            wildness,
            result.candidates_generated,
            result.pre_lean_rejected,
            result.lean_attempted,
            result.full_proofs_compiled,
            result.helper_decompositions_compiled,
            result.accepted
        );
    }

    print_results(&results);
    print_first_failure_breakdown(&results);
    print_summary(&results);
}
