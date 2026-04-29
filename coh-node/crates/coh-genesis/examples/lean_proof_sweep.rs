//! Lean Proof NPE Wildness Sweep
//!
//! Runs a full benchmark sweep measuring formation admission for Lean proof candidates.

use coh_genesis::lean_proof::{
    build_formation_result, compute_genesis_metrics, is_formation_admissible,
    LeanVerificationReport, ProofCandidate, ProofClass, ProofFirstFailure, ProofPolicy,
    ProofSelectorMode,
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

// === Sweep Results ===
#[derive(Clone, Debug, Default)]
struct SweepResult {
    wildness: f64,

    // Counts
    genesis_accept: usize,
    compile_pass: usize,
    proof_clean: usize, // no sorry/admit/axiom
    formation_accept: usize,

    // Novelty
    avg_novelty: f64,
    max_formation_novelty: f64,

    // Margins
    avg_genesis_margin: f64,
    avg_coherence_margin: f64,

    // First failure breakdown
    failure_genesis: usize,
    failure_compiles: usize,
    failure_new_axiom: usize,
    failure_sorry: usize,
    failure_admit: usize,
    failure_statement: usize,
    failure_forbidden: usize,
    failure_coherence: usize,
    failure_accepted: usize,

    avg_safe_score: f64,
}

fn run_sweep_one_level(
    wildness: f64,
    count: usize,
    seed: u32,
    policy: &ProofPolicy,
) -> SweepResult {
    let mut rng = Mulberry32::new(seed);
    let base_complexity = 1000u128;

    let mut result = SweepResult {
        wildness,
        ..Default::default()
    };

    for _ in 0..count {
        let proof_class = ProofClass::select(wildness, rng.next_f64());
        let proof_text = generate_proof_text(&proof_class, &mut rng);
        let tactic_count = 1 + (rng.next_f64() * wildness * 5.0) as usize;
        let helper_lemmas = if wildness >= 1.5 {
            (rng.next_f64() * wildness * 2.0) as usize
        } else {
            0
        };

        // Check for sorry/admit BEFORE moving proof_text into candidate
        let has_sorry = proof_text.contains("sorry");
        let has_admit = proof_text.contains("admit");
        let statement_unchanged = !proof_text.contains("theorem ") || proof_text.len() < 200;

        let candidate = ProofCandidate {
            id: format!("proof-{:?}-{}", proof_class, rng.next()),
            wildness,
            target_theorem: "isRationalInf_add_inf_le".to_string(),
            proof_text,
            proof_tactics: vec![],
            tactic_count,
            helper_lemmas,
            imports: vec![],
            novelty: (wildness + rng.next_f64() * 2.0).min(10.0),
        };

        // Simulate Lean verification
        // Higher wildness = more likely to fail
        let compile_fail = rng.next_f64() < (wildness / 15.0).min(0.95);
        let new_axioms = if wildness >= 5.0 && rng.next_f64() < 0.3 {
            1
        } else {
            0
        };
        let forbidden_import = rng.next_f64() < 0.1;

        let report = LeanVerificationReport {
            compiles: !compile_fail,
            has_sorry,
            has_admit,
            new_axioms,
            statement_unchanged,
            forbidden_imports: forbidden_import,
            build_time_ms: 100 + (wildness * 50.0) as u64,
            errors: vec![],
            warnings: if rng.next_f64() < 0.3 { 1 } else { 0 },
            genesis_margin: 0,
            coherence_margin: 0,
            formation_accept: false,
        };

        let (formation_accept, gen_margin, coh_margin) =
            is_formation_admissible(&candidate, base_complexity, &report);

        // Count stages
        let genesis_accept = gen_margin >= 0;
        if genesis_accept {
            result.genesis_accept += 1;
        }
        if report.compiles {
            result.compile_pass += 1;
        }
        if !has_sorry && !has_admit && new_axioms == 0 {
            result.proof_clean += 1;
        }
        if formation_accept {
            result.formation_accept += 1;
        }

        // First failure
        let failure = ProofFirstFailure::classify(gen_margin, &report, coh_margin);
        match failure {
            ProofFirstFailure::Genesis => result.failure_genesis += 1,
            ProofFirstFailure::Compiles => result.failure_compiles += 1,
            ProofFirstFailure::Sorry => result.failure_sorry += 1,
            ProofFirstFailure::Admit => result.failure_admit += 1,
            ProofFirstFailure::StatementChanged => result.failure_statement += 1,
            ProofFirstFailure::ForbiddenImport => result.failure_forbidden += 1,
            ProofFirstFailure::Coherence => result.failure_coherence += 1,
            ProofFirstFailure::Accepted => result.failure_accepted += 1,
            // NewAxiom maps to generic failure for now
            other => {
                // Treat as compile issue for simplicity
                result.failure_compiles += 1;
            }
        }

        // Stats
        result.avg_novelty += candidate.novelty;
        result.avg_genesis_margin += gen_margin as f64;
        result.avg_coherence_margin += coh_margin as f64;

        if formation_accept {
            result.max_formation_novelty = result.max_formation_novelty.max(candidate.novelty);
        }

        let safe_score = candidate.novelty + 1.0 * gen_margin.min(coh_margin) as f64;
        result.avg_safe_score += safe_score;
    }

    // Averages
    let n = count as f64;
    result.avg_novelty /= n;
    result.avg_genesis_margin /= n;
    result.avg_coherence_margin /= n;
    result.avg_safe_score /= n;

    result
}

fn generate_proof_text(class: &ProofClass, rng: &mut Mulberry32) -> String {
    match class {
        ProofClass::Direct => "exact dec_trivial".to_string(),
        ProofClass::HelperLemma => format!("have h : helper_lemma_{} := by sorry", rng.next()),
        ProofClass::Simplify => "simp only [add_comm]".to_string(),
        ProofClass::Cases => "cases h1".to_string(),
        ProofClass::Induction => "induction on x".to_string(),
        ProofClass::LibrarySearch => "library_search".to_string(),
        ProofClass::Weaken => "exactsorry".to_string(),
        ProofClass::Refactor => "rintro".to_string(),
    }
}

fn print_results(results: &[SweepResult]) {
    println!("\n=== Lean Proof Wildness Sweep ===");
    println!(" λ     Gen    Compile  Clean   Form");
    println!("---   ----   --------  -----   ----");
    for r in results {
        println!(
            "{:4.1} {:5} {:9} {:7} {:6}",
            r.wildness, r.genesis_accept, r.compile_pass, r.proof_clean, r.formation_accept
        );
    }
}

fn print_first_failure(results: &[SweepResult]) {
    println!("\n=== First Failure Distribution ===");
    println!(" λ     Gen  Compile Axiom Sorry Admit State Forbid Coh Accept");
    for r in results {
        println!(
            "{:4.1} {:5} {:7} {:5} {:5} {:5} {:6} {:4} {:6}",
            r.wildness,
            r.failure_genesis,
            r.failure_compiles,
            r.failure_new_axiom,
            r.failure_sorry,
            r.failure_statement,
            r.failure_forbidden,
            r.failure_coherence,
            r.failure_accepted
        );
    }
}

fn main() {
    println!("Lean Proof NPE Wildness Sweep");
    println!("============================");

    let seed = 42;
    let count = 100;
    let levels = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0];

    println!("seed: {}  count: {}  levels: {:?}", seed, count, levels);
    println!();

    let policy = ProofPolicy::default();
    println!(
        "Policy: allow_sorry={}, allow_admit={}, max_tactics={}",
        policy.allow_sorry, policy.allow_admit, policy.max_tactic_count
    );
    println!();

    let mut results = Vec::new();
    for &lambda in &levels {
        let result = run_sweep_one_level(lambda, count, seed + lambda as u32, &policy);
        println!(
            "λ={:.1}: Gen={} Compile={} Clean={} Form={}",
            lambda,
            result.genesis_accept,
            result.compile_pass,
            result.proof_clean,
            result.formation_accept
        );
        results.push(result);
    }

    print_results(&results);
    print_first_failure(&results);

    // Summary
    println!("\n=== Summary ===");
    let mut lambda_star = 0.0;
    let mut max_yield = 0.0;
    for r in &results {
        let rate = r.formation_accept as f64 / count as f64;
        let yield_val = rate * r.avg_novelty;
        if yield_val > max_yield {
            max_yield = yield_val;
            lambda_star = r.wildness;
        }
    }
    println!("λ* (optimal): {:.1}", lambda_star);
    println!("Max yield: {:.2}", max_yield);

    println!("\n=== Complete ===");
}
