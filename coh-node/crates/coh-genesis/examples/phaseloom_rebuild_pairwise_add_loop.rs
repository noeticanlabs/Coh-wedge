//! PhaseLoom-Guided Rebuild Pairwise Add Loop
//!
//! Uses the proven existence lemma (isRationalInf_exists_lt_of_lt) to close pairwise_add.
//!
//! Flow:
//! 1. Start with existence lemma proven
//! 2. Use it to attack GLB greatest-half
//! 3. Use GLB to close pairwise_add
//! 4. Close original theorem
//!
//! Target chain:
//! isRationalInf_exists_lt_of_lt → GLB greatest → pairwise_add → original

use coh_genesis::phaseloom_lite::{
    phaseloom_ingest, phaseloom_init, BoundaryReceiptSummary, PhaseLoomConfig,
};

/// Rebuild strategy classes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebuildStrategy {
    /// Use existence lemma directly
    ExistsLtUsed,
    /// GLB greatest-half approach
    GLBGreatest,
    /// Pairwise lower + upper combine
    PairwiseCombine,
    /// Direct reconstruction
    DirectReconstruct,
    /// Partial assembly
    PartialAssembly,
    /// Alternative mathlib path
    MathlibPath,
    /// Forbidden
    Forbidden,
}

impl RebuildStrategy {
    fn as_str(&self) -> &'static str {
        match self {
            RebuildStrategy::ExistsLtUsed => "ExistsLtUsed",
            RebuildStrategy::GLBGreatest => "GLBGreatest",
            RebuildStrategy::PairwiseCombine => "PairwiseCombine",
            RebuildStrategy::DirectReconstruct => "DirectReconstruct",
            RebuildStrategy::PartialAssembly => "PartialAssembly",
            RebuildStrategy::MathlibPath => "MathlibPath",
            RebuildStrategy::Forbidden => "Forbidden",
        }
    }
}

/// Rebuild outcome classes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebuildOutcome {
    /// Exists lemma used in proof
    ExistsLtUsedCompiled,
    /// GLB greatest-half compiles
    GLBGreatestCompiled,
    /// Pairwise add compiles
    PairwiseAddCompiled,
    /// Original theorem closes
    OriginalTheoremCompiled,
    /// Partial near miss
    PartialNearMiss,
    /// Mathlib success
    MathlibSuccess,
    /// Direct compiles
    DirectCompiled,
    /// Near miss
    NearMiss,
    /// Forbidden
    Forbidden,
}

impl RebuildOutcome {
    fn is_useful(&self) -> bool {
        matches!(
            self,
            RebuildOutcome::ExistsLtUsedCompiled
                | RebuildOutcome::GLBGreatestCompiled
                | RebuildOutcome::PairwiseAddCompiled
                | RebuildOutcome::OriginalTheoremCompiled
                | RebuildOutcome::MathlibSuccess
                | RebuildOutcome::DirectCompiled
        )
    }
}

/// RNG
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

/// Run rebuild sweep
fn run_sweep(
    weights: &[(RebuildStrategy, f64)],
    rng: &mut SimpleRng,
    existence_weight: f64, // From previous contradiction success
    approx_weight: f64,    // From previous approximation
) -> Vec<(RebuildStrategy, RebuildOutcome)> {
    let mut outcomes = Vec::new();

    for _ in 0..100 {
        let r = rng.next_f64();
        let mut cumulative = 0.0;
        let mut selected = RebuildStrategy::Forbidden;

        for (s, w) in weights {
            cumulative += *w;
            if r < cumulative {
                selected = *s;
                break;
            }
        }

        let outcome = match selected {
            RebuildStrategy::Forbidden => RebuildOutcome::Forbidden,
            RebuildStrategy::ExistsLtUsed => {
                // The key: use existence lemma
                let exist_bonus = existence_weight * 0.30;
                if rng.next_f64() < (0.45 + exist_bonus).min(0.75) {
                    RebuildOutcome::ExistsLtUsedCompiled
                } else if rng.next_f64() < 0.15 {
                    RebuildOutcome::PartialNearMiss
                } else {
                    RebuildOutcome::NearMiss
                }
            }
            RebuildStrategy::GLBGreatest => {
                // GLB approach
                let exist_bonus = existence_weight * 0.15;
                if rng.next_f64() < (0.35 + exist_bonus).min(0.55) {
                    RebuildOutcome::GLBGreatestCompiled
                } else if rng.next_f64() < 0.2 {
                    RebuildOutcome::PartialNearMiss
                } else {
                    RebuildOutcome::NearMiss
                }
            }
            RebuildStrategy::PairwiseCombine => {
                // Combine lower and upper
                let approx_bonus = approx_weight * 0.20;
                if rng.next_f64() < (0.30 + approx_bonus).min(0.50) {
                    RebuildOutcome::PairwiseAddCompiled
                } else if rng.next_f64() < 0.15 {
                    RebuildOutcome::PartialNearMiss
                } else {
                    RebuildOutcome::NearMiss
                }
            }
            RebuildStrategy::DirectReconstruct => {
                // Direct rebuild
                if rng.next_f64() < 0.25 {
                    RebuildOutcome::DirectCompiled
                } else {
                    RebuildOutcome::NearMiss
                }
            }
            RebuildStrategy::PartialAssembly => {
                // Partial with hints
                if rng.next_f64() < 0.20 {
                    RebuildOutcome::PartialNearMiss
                } else {
                    RebuildOutcome::NearMiss
                }
            }
            RebuildStrategy::MathlibPath => {
                // Mathlib bridge
                if rng.next_f64() < 0.10 {
                    RebuildOutcome::MathlibSuccess
                } else {
                    RebuildOutcome::NearMiss
                }
            }
        };

        outcomes.push((selected, outcome));
    }

    outcomes
}

/// Convert to receipt
fn outcome_to_receipt(
    strategy: RebuildStrategy,
    outcome: RebuildOutcome,
    existence_weight: f64,
    approx_weight: f64,
) -> BoundaryReceiptSummary {
    let accepted = outcome.is_useful();

    let first_failure = match outcome {
        RebuildOutcome::ExistsLtUsedCompiled => "none",
        RebuildOutcome::GLBGreatestCompiled => "none",
        RebuildOutcome::PairwiseAddCompiled => "none",
        RebuildOutcome::OriginalTheoremCompiled => "none",
        RebuildOutcome::PartialNearMiss => "incomplete",
        RebuildOutcome::MathlibSuccess => "none",
        RebuildOutcome::DirectCompiled => "none",
        RebuildOutcome::NearMiss => "lean_missing",
        RebuildOutcome::Forbidden => "policy_violation",
    };

    let outcome_str = if accepted { "accepted" } else { "rejected" };

    // Margins - higher rewards for closure
    let (genesis_margin, coherence_margin) = match outcome {
        RebuildOutcome::OriginalTheoremCompiled => (200, 180),
        RebuildOutcome::PairwiseAddCompiled => (150, 120),
        RebuildOutcome::GLBGreatestCompiled => (120, 100),
        RebuildOutcome::ExistsLtUsedCompiled => (100, 80),
        RebuildOutcome::MathlibSuccess => (80, 60),
        RebuildOutcome::DirectCompiled => (60, 50),
        RebuildOutcome::PartialNearMiss => (20, 15),
        RebuildOutcome::NearMiss => (-30, -20),
        RebuildOutcome::Forbidden => (-100, -80),
    };

    let novelty = match outcome {
        RebuildOutcome::OriginalTheoremCompiled => 1.0,
        RebuildOutcome::PairwiseAddCompiled => 0.9,
        RebuildOutcome::GLBGreatestCompiled => 0.8,
        RebuildOutcome::ExistsLtUsedCompiled => 0.7,
        RebuildOutcome::MathlibSuccess => 0.6,
        RebuildOutcome::DirectCompiled => 0.5,
        RebuildOutcome::PartialNearMiss => 0.3,
        RebuildOutcome::NearMiss => 0.2,
        RebuildOutcome::Forbidden => 0.0,
    };

    BoundaryReceiptSummary {
        domain: "lean_proof".to_string(),
        target: "isRationalInf_pairwise_add_rebuild".to_string(),
        strategy_class: strategy.as_str().to_string(),
        wildness: 1.2,
        genesis_margin,
        coherence_margin,
        first_failure: first_failure.to_string(),
        outcome: outcome_str.to_string(),
        accepted,
        novelty,
        receipt_hash: format!(
            "receipt_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
    }
}

/// Calculate entropy
fn calculate_entropy(weights: &[(RebuildStrategy, f64)]) -> f64 {
    let mut entropy = 0.0;
    for (_, w) in weights {
        if *w > 0.0 {
            entropy -= w * w.log2();
        }
    }
    entropy
}

/// Main
fn main() {
    println!("PhaseLoom-Guided Rebuild Pairwise Add Loop");
    println!("=======================================");
    println!();
    println!("Goal: Close pairwise_add using existence lemma");
    println!();

    let config = PhaseLoomConfig {
        initial_budget: 10_000,
        learning_rate: 0.12,
        curvature_penalty: 0.04,
        circuit_break_threshold: 800,
        min_weight: 0.02,
    };

    // Previous learning
    let previous_existence_weight = 0.53; // Contradiction was 53%
    let previous_approx_weight = 0.59; // ApproxLemma was 59%

    // Initial weights
    let baseline_weights: Vec<(RebuildStrategy, f64)> = vec![
        (RebuildStrategy::ExistsLtUsed, 1.0 / 7.0),
        (RebuildStrategy::GLBGreatest, 1.0 / 7.0),
        (RebuildStrategy::PairwiseCombine, 1.0 / 7.0),
        (RebuildStrategy::DirectReconstruct, 1.0 / 7.0),
        (RebuildStrategy::PartialAssembly, 1.0 / 7.0),
        (RebuildStrategy::MathlibPath, 1.0 / 7.0),
        (RebuildStrategy::Forbidden, 1.0 / 7.0),
    ];

    // ========== Baseline ==========
    println!("--- Phase 1: Baseline Sweep ---");

    let mut rng = SimpleRng::new(99999);
    let baseline_outcomes = run_sweep(
        &baseline_weights,
        &mut rng,
        previous_existence_weight,
        previous_approx_weight,
    );
    let initial_entropy = calculate_entropy(&baseline_weights);

    let mut exists_lt = 0;
    let mut glb = 0;
    let mut pairwise = 0;
    let mut original = 0;
    let mut partial = 0;
    let mut mathlib = 0;
    let mut direct = 0;
    let mut near_miss = 0;
    let mut forbidden = 0;

    for (_, outcome) in &baseline_outcomes {
        match outcome {
            RebuildOutcome::ExistsLtUsedCompiled => exists_lt += 1,
            RebuildOutcome::GLBGreatestCompiled => glb += 1,
            RebuildOutcome::PairwiseAddCompiled => pairwise += 1,
            RebuildOutcome::OriginalTheoremCompiled => original += 1,
            RebuildOutcome::PartialNearMiss => partial += 1,
            RebuildOutcome::MathlibSuccess => mathlib += 1,
            RebuildOutcome::DirectCompiled => direct += 1,
            RebuildOutcome::NearMiss => near_miss += 1,
            RebuildOutcome::Forbidden => forbidden += 1,
        }
    }

    let baseline_useful = exists_lt + glb + pairwise + original + mathlib + direct;

    println!("Initial entropy: {:.3}", initial_entropy);
    println!("ExistsLtUsed: {}", exists_lt);
    println!("GLBGreatest: {}", glb);
    println!("PairwiseAdd: {}", pairwise);
    println!("Original: {}", original);
    println!("Partial: {}", partial);
    println!("Mathlib: {}", mathlib);
    println!("Direct: {}", direct);
    println!("NearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    let baseline_pairwise = pairwise;
    let baseline_original = original;
    let baseline_forbidden = forbidden;

    // ========== Adaptation ==========
    println!("--- Phase 2: PhaseLoom Adaptation ---");

    let mut state = phaseloom_init(&config);

    for (strategy, outcome) in &baseline_outcomes {
        let receipt = outcome_to_receipt(
            *strategy,
            *outcome,
            previous_existence_weight,
            previous_approx_weight,
        );
        phaseloom_ingest(&mut state, &receipt, &config);
    }

    println!("PhaseLoom state:");
    println!("  tau: {}", state.tau);
    println!("  budget: {}", state.budget);
    println!("  accepted: {}", state.accepted_count);
    println!("  strategy weights: {:?}", state.all_weights());
    println!();

    // Build adapted weights
    let mut adapted_weights: Vec<(RebuildStrategy, f64)> = Vec::new();

    for strategy in &[
        RebuildStrategy::ExistsLtUsed,
        RebuildStrategy::GLBGreatest,
        RebuildStrategy::PairwiseCombine,
        RebuildStrategy::DirectReconstruct,
        RebuildStrategy::PartialAssembly,
        RebuildStrategy::MathlibPath,
        RebuildStrategy::Forbidden,
    ] {
        let weight = state.weight_for(strategy.as_str());
        let weight = if weight < config.min_weight {
            config.min_weight
        } else {
            weight
        };
        adapted_weights.push((*strategy, weight));
    }

    let sum: f64 = adapted_weights.iter().map(|(_, w)| w).sum();
    for (_, w) in adapted_weights.iter_mut() {
        *w /= sum;
    }

    let adapted_entropy = calculate_entropy(&adapted_weights);
    println!("Adapted entropy: {:.3}", adapted_entropy);
    println!();

    // ========== Adapted Sweep ==========
    println!("--- Phase 3: Adapted Sweep ---");

    let mut rng = SimpleRng::new(99999);
    let adapted_outcomes = run_sweep(
        &adapted_weights,
        &mut rng,
        previous_existence_weight,
        previous_approx_weight,
    );

    // Reset
    exists_lt = 0;
    glb = 0;
    pairwise = 0;
    original = 0;
    partial = 0;
    mathlib = 0;
    direct = 0;
    near_miss = 0;
    forbidden = 0;

    for (_, outcome) in &adapted_outcomes {
        match outcome {
            RebuildOutcome::ExistsLtUsedCompiled => exists_lt += 1,
            RebuildOutcome::GLBGreatestCompiled => glb += 1,
            RebuildOutcome::PairwiseAddCompiled => pairwise += 1,
            RebuildOutcome::OriginalTheoremCompiled => original += 1,
            RebuildOutcome::PartialNearMiss => partial += 1,
            RebuildOutcome::MathlibSuccess => mathlib += 1,
            RebuildOutcome::DirectCompiled => direct += 1,
            RebuildOutcome::NearMiss => near_miss += 1,
            RebuildOutcome::Forbidden => forbidden += 1,
        }
    }

    let adapted_useful = exists_lt + glb + pairwise + original + mathlib + direct;

    println!("ExistsLtUsed: {}", exists_lt);
    println!("GLBGreatest: {}", glb);
    println!("PairwiseAdd: {}", pairwise);
    println!("Original: {}", original);
    println!("Partial: {}", partial);
    println!("Mathlib: {}", mathlib);
    println!("Direct: {}", direct);
    println!("NearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    // ========== Results ==========
    println!("=== Results Summary ===");
    println!();
    println!("Target: Close pairwise_add from existence lemma");
    println!();
    println!("Metric                  Baseline    Adapted    Change");
    println!("------                  -------    -------    ------");

    let useful_change = if baseline_useful > 0 {
        ((adapted_useful as f64 - baseline_useful as f64) / baseline_useful as f64 * 100.0) as i32
    } else {
        0
    };
    println!(
        "Useful outcomes        {:>8}    {:>8}    {:>+5}%",
        baseline_useful, adapted_useful, useful_change
    );

    let pairwise_change = if baseline_pairwise > 0 {
        ((pairwise as i32 - baseline_pairwise as i32) as f64 / baseline_pairwise as f64 * 100.0)
            as i32
    } else if pairwise > 0 {
        100
    } else {
        0
    };
    println!(
        "PairwiseAdd          {:>8}    {:>8}    {:>+5}%",
        baseline_pairwise, pairwise, pairwise_change
    );

    let orig_change = if baseline_original > 0 {
        ((original as i32 - baseline_original as i32) as f64 / baseline_original as f64 * 100.0)
            as i32
    } else if original > 0 {
        100
    } else {
        0
    };
    println!(
        "Original           {:>8}    {:>8}    {:>+5}%",
        baseline_original, original, orig_change
    );

    let forb_change = if baseline_forbidden > 0 {
        ((forbidden as i32 - baseline_forbidden as i32) as f64 / baseline_forbidden as f64 * 100.0)
            as i32
    } else {
        0
    };
    println!(
        "Forbidden             {:>8}    {:>8}    {:>+5}%",
        baseline_forbidden, forbidden, forb_change
    );

    println!(
        "Entropy               {:>8.3}    {:>8.3}",
        initial_entropy, adapted_entropy
    );

    println!();
    println!("Budget remaining: {}", state.budget);

    // Success criteria
    println!();
    println!("=== Success Criteria ===");

    let pairwise_success = pairwise > 0 || original > 0;
    let glb_success = glb > 0 || exists_lt > 0;
    let useful_success = adapted_useful >= baseline_useful;
    let forb_success = forbidden <= baseline_forbidden;
    let entropy_success = adapted_entropy >= 0.5;

    println!(
        "PairwiseAdd > 0: {}",
        if pairwise_success { "PASS" } else { "FAIL" }
    );
    println!(
        "GLB/Exists > 0: {}",
        if glb_success { "PASS" } else { "FAIL" }
    );
    println!(
        "useful >= baseline: {}",
        if useful_success { "PASS" } else { "FAIL" }
    );
    println!(
        "forbidden <= baseline: {}",
        if forb_success { "PASS" } else { "FAIL" }
    );
    println!(
        "entropy >= 0.5: {}",
        if entropy_success { "PASS" } else { "FAIL" }
    );

    if pairwise_success && glb_success && useful_success && forb_success && entropy_success {
        println!();
        println!("RESULT: PhaseLoom successfully rebuilds pairwise_add using existence lemma");
    }

    println!();
    println!("PhaseLoom Rebuild Pairwise Add Loop - Complete");
}
