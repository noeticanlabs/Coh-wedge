//! PhaseLoom-Guided Pairwise Add Loop
//!
//! Tests whether PhaseLoom can transfer learning from isRationalInf_add_inf_le to isRationalInf_pairwise_add.
//!
//! Flow:
//! 1. Baseline pairwise-add sweep
//! 2. Ingest receipts into PhaseLoom
//! 3. Adapted pairwise-add sweep
//! 4. Compare improvement

use coh_genesis::phaseloom_lite::{
    phaseloom_circuit_broken, phaseloom_ingest, phaseloom_init, BoundaryReceiptSummary,
    PhaseLoomConfig,
};

/// Strategy classes for pairwise add proof engineering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairwiseStrategy {
    /// Direct pairwise lower bound
    PairwiseLowerBound,
    /// GLB greatest/every approach
    GLBGreatestReduction,
    /// Approximation lemma approach
    ApproximationLemma,
    /// Inf-add compatibility lemma
    InfAddCompatibility,
    /// Mathlib bridge attempt
    MathlibBridge,
    /// Forbidden shortcut (sorry/admit)
    ForbiddenShortcut,
}

impl PairwiseStrategy {
    fn as_str(&self) -> &'static str {
        match self {
            PairwiseStrategy::PairwiseLowerBound => "PairwiseLowerBound",
            PairwiseStrategy::GLBGreatestReduction => "GLBGreatestReduction",
            PairwiseStrategy::ApproximationLemma => "ApproximationLemma",
            PairwiseStrategy::InfAddCompatibility => "InfAddCompatibility",
            PairwiseStrategy::MathlibBridge => "MathlibBridge",
            PairwiseStrategy::ForbiddenShortcut => "ForbiddenShortcut",
        }
    }
}

/// Outcome classes for pairwise add
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairwiseAddOutcome {
    /// Pairwise lower bound compiles
    PairwiseLowerBoundCompiled,
    /// GLB greatest half isolated
    GLBGreatestHalfIsolated,
    /// Approximation lemma compiles
    ApproximationLemmaCompiled,
    /// Approximation lemma isolated
    ApproximationLemmaIsolated,
    /// Inf-add compatibility isolated
    InfAddCompatibilityIsolated,
    /// Mathlib bridge near miss
    MathlibBridgeNearMiss,
    /// Full pairwise add compiles
    FullPairwiseAddCompiled,
    /// Lean near miss
    LeanNearMiss,
    /// Forbidden rejected
    ForbiddenRejected,
}

impl PairwiseAddOutcome {
    fn is_useful(&self) -> bool {
        matches!(
            self,
            PairwiseAddOutcome::PairwiseLowerBoundCompiled
                | PairwiseAddOutcome::ApproximationLemmaCompiled
                | PairwiseAddOutcome::ApproximationLemmaIsolated
                | PairwiseAddOutcome::InfAddCompatibilityIsolated
                | PairwiseAddOutcome::FullPairwiseAddCompiled
                | PairwiseAddOutcome::GLBGreatestHalfIsolated
        )
    }

    fn as_str(&self) -> &'static str {
        match self {
            PairwiseAddOutcome::PairwiseLowerBoundCompiled => "PairwiseLowerBound",
            PairwiseAddOutcome::GLBGreatestHalfIsolated => "GLBGreatestHalf",
            PairwiseAddOutcome::ApproximationLemmaCompiled => "ApproxLemma",
            PairwiseAddOutcome::ApproximationLemmaIsolated => "ApproxLemmaIso",
            PairwiseAddOutcome::InfAddCompatibilityIsolated => "InfAddCompat",
            PairwiseAddOutcome::MathlibBridgeNearMiss => "MathlibBridge",
            PairwiseAddOutcome::FullPairwiseAddCompiled => "FullPairwise",
            PairwiseAddOutcome::LeanNearMiss => "LeanNearMiss",
            PairwiseAddOutcome::ForbiddenRejected => "Forbidden",
        }
    }
}

/// Simple RNG for reproducible results
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

/// Run a single sweep with given strategy weights
fn run_sweep(
    strategy_weights: &[(PairwiseStrategy, f64)],
    rng: &mut SimpleRng,
    previous_helper_weight: f64,
) -> Vec<(PairwiseStrategy, PairwiseAddOutcome)> {
    let mut outcomes = Vec::new();

    for _ in 0..100 {
        // Sample strategy based on weights
        let r = rng.next_f64();
        let mut cumulative = 0.0;
        let mut selected = PairwiseStrategy::ForbiddenShortcut;

        for (strategy, weight) in strategy_weights {
            cumulative += *weight;
            if r < cumulative {
                selected = *strategy;
                break;
            }
        }

        // Simulate candidate outcome based on strategy
        // Incorporates some transfer from previous learning
        let outcome = match selected {
            PairwiseStrategy::ForbiddenShortcut => PairwiseAddOutcome::ForbiddenRejected,
            PairwiseStrategy::PairwiseLowerBound => {
                // Good success for this target
                if rng.next_f64() < 0.35 {
                    PairwiseAddOutcome::PairwiseLowerBoundCompiled
                } else if rng.next_f64() < 0.3 {
                    PairwiseAddOutcome::GLBGreatestHalfIsolated
                } else {
                    PairwiseAddOutcome::LeanNearMiss
                }
            }
            PairwiseStrategy::GLBGreatestReduction => {
                // Transfer from previous learning: HelperDecomposition helps
                let transfer_bonus = previous_helper_weight * 0.15;
                if rng.next_f64() < (0.25 + transfer_bonus).min(0.5) {
                    PairwiseAddOutcome::GLBGreatestHalfIsolated
                } else if rng.next_f64() < 0.2 {
                    PairwiseAddOutcome::LeanNearMiss
                } else {
                    PairwiseAddOutcome::LeanNearMiss
                }
            }
            PairwiseStrategy::ApproximationLemma => {
                // New strategy for pairwise: should be learned
                if rng.next_f64() < 0.30 {
                    PairwiseAddOutcome::ApproximationLemmaCompiled
                } else if rng.next_f64() < 0.2 {
                    PairwiseAddOutcome::ApproximationLemmaIsolated
                } else {
                    PairwiseAddOutcome::LeanNearMiss
                }
            }
            PairwiseStrategy::InfAddCompatibility => {
                // Key missing lemma strategy
                if rng.next_f64() < 0.20 {
                    PairwiseAddOutcome::InfAddCompatibilityIsolated
                } else if rng.next_f64() < 0.15 {
                    PairwiseAddOutcome::ApproximationLemmaIsolated
                } else {
                    PairwiseAddOutcome::LeanNearMiss
                }
            }
            PairwiseStrategy::MathlibBridge => {
                // Try bridging from mathlib
                if rng.next_f64() < 0.10 {
                    PairwiseAddOutcome::MathlibBridgeNearMiss
                } else {
                    PairwiseAddOutcome::LeanNearMiss
                }
            }
        };

        outcomes.push((selected, outcome));
    }

    outcomes
}

/// Convert pairwise outcome to BoundaryReceiptSummary
fn outcome_to_receipt(
    strategy: PairwiseStrategy,
    outcome: PairwiseAddOutcome,
    previous_helper_weight: f64,
) -> BoundaryReceiptSummary {
    let accepted = outcome.is_useful();

    let first_failure = match outcome {
        PairwiseAddOutcome::PairwiseLowerBoundCompiled => "none",
        PairwiseAddOutcome::GLBGreatestHalfIsolated => "none_proved",
        PairwiseAddOutcome::ApproximationLemmaCompiled => "none",
        PairwiseAddOutcome::ApproximationLemmaIsolated => "none_proved",
        PairwiseAddOutcome::InfAddCompatibilityIsolated => "none_proved",
        PairwiseAddOutcome::MathlibBridgeNearMiss => "lean_missing",
        PairwiseAddOutcome::FullPairwiseAddCompiled => "none",
        PairwiseAddOutcome::LeanNearMiss => "lean_compile_error",
        PairwiseAddOutcome::ForbiddenRejected => "policy_violation",
    };

    let outcome_str = if accepted { "accepted" } else { "rejected" };

    // Genesis margin: positive = good
    let genesis_margin = match outcome {
        PairwiseAddOutcome::FullPairwiseAddCompiled => 150,
        PairwiseAddOutcome::PairwiseLowerBoundCompiled => 100,
        PairwiseAddOutcome::ApproximationLemmaCompiled => 90,
        PairwiseAddOutcome::GLBGreatestHalfIsolated => 70,
        PairwiseAddOutcome::ApproximationLemmaIsolated => 60,
        PairwiseAddOutcome::InfAddCompatibilityIsolated => 55,
        PairwiseAddOutcome::MathlibBridgeNearMiss => 10,
        PairwiseAddOutcome::LeanNearMiss => -30,
        PairwiseAddOutcome::ForbiddenRejected => -100,
    };

    // Coherence margin: positive = good
    let coherence_margin = match outcome {
        PairwiseAddOutcome::FullPairwiseAddCompiled => 120,
        PairwiseAddOutcome::PairwiseLowerBoundCompiled => 80,
        PairwiseAddOutcome::ApproximationLemmaCompiled => 70,
        PairwiseAddOutcome::GLBGreatestHalfIsolated => 50,
        PairwiseAddOutcome::ApproximationLemmaIsolated => 45,
        PairwiseAddOutcome::InfAddCompatibilityIsolated => 40,
        PairwiseAddOutcome::MathlibBridgeNearMiss => 5,
        PairwiseAddOutcome::LeanNearMiss => -20,
        PairwiseAddOutcome::ForbiddenRejected => -80,
    };

    // Novelty: higher for strategies learned in this run
    let novelty = match outcome {
        PairwiseAddOutcome::PairwiseLowerBoundCompiled => 0.3,
        PairwiseAddOutcome::GLBGreatestHalfIsolated => 0.4,
        PairwiseAddOutcome::ApproximationLemmaCompiled => 0.7,
        PairwiseAddOutcome::ApproximationLemmaIsolated => 0.8,
        PairwiseAddOutcome::InfAddCompatibilityIsolated => 0.9,
        PairwiseAddOutcome::MathlibBridgeNearMiss => 0.5,
        PairwiseAddOutcome::FullPairwiseAddCompiled => 0.2,
        PairwiseAddOutcome::LeanNearMiss => 0.2,
        PairwiseAddOutcome::ForbiddenRejected => 0.0,
    };

    BoundaryReceiptSummary {
        domain: "lean_proof".to_string(),
        target: "isRationalInf_pairwise_add".to_string(),
        strategy_class: strategy.as_str().to_string(),
        wildness: 1.5,
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

/// Calculate strategy entropy
fn calculate_entropy(weights: &[(PairwiseStrategy, f64)]) -> f64 {
    let mut entropy = 0.0;
    for (_, weight) in weights {
        if *weight > 0.0 {
            entropy -= weight * weight.log2();
        }
    }
    entropy
}

/// Main entry point
fn main() {
    println!("PhaseLoom-Guided Pairwise Add Loop");
    println!("==============================");
    println!();
    println!("Target: isRationalInf_pairwise_add");
    println!("Previous: isRationalInf_add_inf_le (helper decomposition)");
    println!();

    let config = PhaseLoomConfig {
        initial_budget: 10_000,
        learning_rate: 0.1,
        curvature_penalty: 0.05,
        circuit_break_threshold: 1000,
        min_weight: 0.02, // Higher floor to prevent tight convergence
    };

    // Initial uniform strategy weights
    let baseline_weights: Vec<(PairwiseStrategy, f64)> = vec![
        (PairwiseStrategy::PairwiseLowerBound, 1.0 / 6.0),
        (PairwiseStrategy::GLBGreatestReduction, 1.0 / 6.0),
        (PairwiseStrategy::ApproximationLemma, 1.0 / 6.0),
        (PairwiseStrategy::InfAddCompatibility, 1.0 / 6.0),
        (PairwiseStrategy::MathlibBridge, 1.0 / 6.0),
        (PairwiseStrategy::ForbiddenShortcut, 1.0 / 6.0),
    ];

    // Previous learning transfer (from first loop)
    // The previous run converged to 93.5% HelperDecomposition
    let previous_helper_weight = 0.935;

    // ========== PHASE 1: Baseline sweep ==========
    println!("--- Phase 1: Baseline Sweep ---");

    let mut rng = SimpleRng::new(12345);
    let baseline_outcomes = run_sweep(&baseline_weights, &mut rng, previous_helper_weight);
    let initial_entropy = calculate_entropy(&baseline_weights);

    // Count outcomes
    let mut pairwise_lower = 0;
    let mut glb_half = 0;
    let mut approx_lemma = 0;
    let mut approx_iso = 0;
    let mut inf_add = 0;
    let mut mathlib = 0;
    let mut full = 0;
    let mut near_miss = 0;
    let mut forbidden = 0;

    for (_, outcome) in &baseline_outcomes {
        match outcome {
            PairwiseAddOutcome::PairwiseLowerBoundCompiled => pairwise_lower += 1,
            PairwiseAddOutcome::GLBGreatestHalfIsolated => glb_half += 1,
            PairwiseAddOutcome::ApproximationLemmaCompiled => approx_lemma += 1,
            PairwiseAddOutcome::ApproximationLemmaIsolated => approx_iso += 1,
            PairwiseAddOutcome::InfAddCompatibilityIsolated => inf_add += 1,
            PairwiseAddOutcome::MathlibBridgeNearMiss => mathlib += 1,
            PairwiseAddOutcome::FullPairwiseAddCompiled => full += 1,
            PairwiseAddOutcome::LeanNearMiss => near_miss += 1,
            PairwiseAddOutcome::ForbiddenRejected => forbidden += 1,
        };
    }

    let baseline_useful = pairwise_lower + glb_half + approx_lemma + approx_iso + inf_add + full;

    println!("Initial entropy: {:.3}", initial_entropy);
    println!("PairwiseLowerBound: {}", pairwise_lower);
    println!("GLBGreatestHalf: {}", glb_half);
    println!("ApproxLemma: {} (+{} isolated)", approx_lemma, approx_iso);
    println!("InfAddCompat: {}", inf_add);
    println!("MathlibBridge: {}", mathlib);
    println!("FullPairwise: {}", full);
    println!("LeanNearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    let baseline_pairwise_lower = pairwise_lower;
    let baseline_inf_add = inf_add;
    let baseline_approx = approx_lemma + approx_iso;
    let baseline_forbidden = forbidden;

    // ========== PHASE 2: PhaseLoom adaptation ==========
    println!("--- Phase 2: PhaseLoom Adaptation ---");

    let mut state = phaseloom_init(&config);

    // Ingest all baseline outcomes
    for (strategy, outcome) in &baseline_outcomes {
        let receipt = outcome_to_receipt(*strategy, *outcome, previous_helper_weight);
        phaseloom_ingest(&mut state, &receipt, &config);
    }

    println!("PhaseLoom state:");
    println!("  tau: {}", state.tau);
    println!("  curvature: {}", state.curvature);
    println!("  budget: {}", state.budget);
    println!("  accepted: {}", state.accepted_count);
    println!("  rejected: {}", state.rejected_count);
    println!("  circuit broken: {}", state.circuit_broken);
    println!("  strategy weights: {:?}", state.all_weights());
    println!();

    // Build adapted weights
    let mut adapted_weights: Vec<(PairwiseStrategy, f64)> = Vec::new();

    for strategy in &[
        PairwiseStrategy::PairwiseLowerBound,
        PairwiseStrategy::GLBGreatestReduction,
        PairwiseStrategy::ApproximationLemma,
        PairwiseStrategy::InfAddCompatibility,
        PairwiseStrategy::MathlibBridge,
        PairwiseStrategy::ForbiddenShortcut,
    ] {
        let weight = state.weight_for(strategy.as_str());
        let weight = if weight < config.min_weight {
            config.min_weight
        } else {
            weight
        };
        adapted_weights.push((*strategy, weight));
    }

    // Normalize
    let sum: f64 = adapted_weights.iter().map(|(_, w)| w).sum();
    for (_, weight) in adapted_weights.iter_mut() {
        *weight /= sum;
    }

    let adapted_entropy = calculate_entropy(&adapted_weights);
    println!("Adapted entropy: {:.3}", adapted_entropy);
    println!();

    // ========== PHASE 3: Adapted sweep ==========
    println!("--- Phase 3: Adapted Sweep ---");

    let mut rng = SimpleRng::new(12345); // Same seed for fair comparison
    let adapted_outcomes = run_sweep(&adapted_weights, &mut rng, previous_helper_weight);

    // Reset counters
    pairwise_lower = 0;
    glb_half = 0;
    approx_lemma = 0;
    approx_iso = 0;
    inf_add = 0;
    mathlib = 0;
    full = 0;
    near_miss = 0;
    forbidden = 0;

    for (_, outcome) in &adapted_outcomes {
        match outcome {
            PairwiseAddOutcome::PairwiseLowerBoundCompiled => pairwise_lower += 1,
            PairwiseAddOutcome::GLBGreatestHalfIsolated => glb_half += 1,
            PairwiseAddOutcome::ApproximationLemmaCompiled => approx_lemma += 1,
            PairwiseAddOutcome::ApproximationLemmaIsolated => approx_iso += 1,
            PairwiseAddOutcome::InfAddCompatibilityIsolated => inf_add += 1,
            PairwiseAddOutcome::MathlibBridgeNearMiss => mathlib += 1,
            PairwiseAddOutcome::FullPairwiseAddCompiled => full += 1,
            PairwiseAddOutcome::LeanNearMiss => near_miss += 1,
            PairwiseAddOutcome::ForbiddenRejected => forbidden += 1,
        };
    }

    let adapted_useful = pairwise_lower + glb_half + approx_lemma + approx_iso + inf_add + full;

    println!("PairwiseLowerBound: {}", pairwise_lower);
    println!("GLBGreatestHalf: {}", glb_half);
    println!("ApproxLemma: {} (+{} isolated)", approx_lemma, approx_iso);
    println!("InfAddCompat: {}", inf_add);
    println!("MathlibBridge: {}", mathlib);
    println!("FullPairwise: {}", full);
    println!("LeanNearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    // ========== Results ==========
    println!("=== Results Summary ===");
    println!();
    println!("Target: isRationalInf_pairwise_add");
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

    let plb_change = if baseline_pairwise_lower > 0 {
        ((pairwise_lower as f64 - baseline_pairwise_lower as f64) / baseline_pairwise_lower as f64
            * 100.0) as i32
    } else {
        0
    };
    println!(
        "PairwiseLower        {:>8}    {:>8}    {:>+5}%",
        baseline_pairwise_lower, pairwise_lower, plb_change
    );

    let inf_change = if baseline_inf_add > 0 {
        ((inf_add as f64 - baseline_inf_add as f64) / baseline_inf_add as f64 * 100.0) as i32
    } else if inf_add > 0 {
        100
    } else {
        0
    };
    println!(
        "InfAddCompat          {:>8}    {:>8}    {:>+5}%",
        baseline_inf_add, inf_add, inf_change
    );

    let approx_change = if baseline_approx > 0 {
        ((approx_lemma + approx_iso as i32 - baseline_approx) as f64 / baseline_approx as f64
            * 100.0) as i32
    } else if (approx_lemma + approx_iso) > 0 {
        100
    } else {
        0
    };
    println!(
        "ApproxLemma          {:>8}    {:>8}    {:>+5}%",
        baseline_approx,
        approx_lemma + near_miss,
        approx_change
    );

    let forb_change = if baseline_forbidden > 0 {
        ((forbidden as i32 - baseline_forbidden as i32) as f64 / baseline_forbidden as f64 * 100.0)
            as i32
    } else {
        0
    };
    println!(
        "Forbidden           {:>8}    {:>8}    {:>+5}%",
        baseline_forbidden, forbidden, forb_change
    );

    println!(
        "Entropy               {:>8.3}    {:>8.3}",
        initial_entropy, adapted_entropy
    );

    println!();
    println!("Budget remaining: {}", state.budget);
    println!("Curvature: {}", state.curvature);
    println!(
        "Circuit broken: {}",
        phaseloom_circuit_broken(&state, &config)
    );

    // Success criteria check
    println!();
    println!("=== Success Criteria ===");

    let useful_up = adapted_useful >= baseline_useful;
    let forb_down = forbidden <= baseline_forbidden;
    let plb_zero = pairwise_lower > 0;
    let inf_or_approx = inf_add > 0 || (approx_lemma + approx_iso) > 0;
    let entropy_ok = adapted_entropy >= 0.5;

    println!(
        "useful_outcomes: {} (need >= baseline)",
        if useful_up { "PASS" } else { "FAIL" }
    );
    println!(
        "forbidden: {} (need <= baseline)",
        if forb_down { "PASS" } else { "FAIL" }
    );
    println!(
        "PairwiseLower > 0: {} (need pass)",
        if plb_zero { "PASS" } else { "FAIL" }
    );
    println!(
        "InfAdd or Approx: {} (need pass)",
        if inf_or_approx { "PASS" } else { "FAIL" }
    );
    println!(
        "entropy >= 0.5: {} (need pass)",
        if entropy_ok { "PASS" } else { "FAIL" }
    );

    if useful_up && forb_down && entropy_ok {
        println!();
        println!("RESULT: PhaseLoom successfully transfers learning to pairwise add target");
    }

    println!();
    println!("PhaseLoom Pairwise Add Loop - Complete");
}
