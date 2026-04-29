//! PhaseLoom-Guided Lean V3 Proof Engineering Loop
//!
//! Demonstrates PhaseLoomLite adaptive memory in the Lean proof engineering domain.
//!
//! Flow: Run two sweeps - baseline (uniform) then adapted (PhaseLoom-informed)
//!
//! This example:
//! 1. Runs baseline sweep with uniform strategy weights
//! 2. Converts outcomes to BoundaryReceiptSummary
//! 3. Ingests receipts into PhaseLoom
//! 4. Runs second sweep with adapted strategy weights
//! 5. Reports before/after improvement

use coh_genesis::phaseloom_lite::{
    phaseloom_circuit_broken, phaseloom_ingest, phaseloom_init, phaseloom_sample,
    BoundaryReceiptSummary, PhaseLoomConfig, PhaseLoomState,
};

/// Strategy classes for Lean proof engineering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeanStrategy {
    /// Direct GLB approach (low success)
    DirectGLB,
    /// Pairwise sum set construction
    PairwiseSumSet,
    /// Helper lemma decomposition
    HelperDecomposition,
    /// Named missing lemma
    NamedMissingLemma,
    /// Library search
    LibrarySearch,
    /// Forbidden shortcut (sorry/admit/axiom)
    ForbiddenShortcut,
}

impl LeanStrategy {
    fn from_class(class: u8) -> Self {
        match class {
            0 => LeanStrategy::DirectGLB,
            1 => LeanStrategy::PairwiseSumSet,
            2 => LeanStrategy::HelperDecomposition,
            3 => LeanStrategy::NamedMissingLemma,
            4 => LeanStrategy::LibrarySearch,
            _ => LeanStrategy::ForbiddenShortcut,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LeanStrategy::DirectGLB => "DirectGLB",
            LeanStrategy::PairwiseSumSet => "PairwiseSumSet",
            LeanStrategy::HelperDecomposition => "HelperDecomposition",
            LeanStrategy::NamedMissingLemma => "NamedMissingLemma",
            LeanStrategy::LibrarySearch => "LibrarySearch",
            LeanStrategy::ForbiddenShortcut => "ForbiddenShortcut",
        }
    }
}

/// Outcome classes for Lean proof candidates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeanOutcome {
    /// Full original proof compiled
    FullOriginalProof,
    /// Helper reduction compiled
    HelperReductionCompiled,
    /// Missing lemma isolated (names theorem but doesn't prove)
    MissingLemmaIsolated,
    /// Reached Lean but failed with useful error
    LeanNearMiss,
    /// Coh blocked (sorry/admit/axiom/statement change)
    ForbiddenRejected,
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

    fn next_u8(&mut self) -> u8 {
        (self.next() >> 56) as u8
    }
}

/// Run a single sweep and return outcomes
fn run_sweep(
    wildness: f64,
    strategy_weights: &[(LeanStrategy, f64)],
    rng: &mut SimpleRng,
) -> Vec<(LeanStrategy, LeanOutcome)> {
    let mut outcomes = Vec::new();

    for _ in 0..100 {
        // Sample strategy based on weights
        let r = rng.next_f64();
        let mut cumulative = 0.0;
        let mut selected = LeanStrategy::ForbiddenShortcut;

        for (strategy, weight) in strategy_weights {
            cumulative += *weight;
            if r < cumulative {
                selected = *strategy;
                break;
            }
        }

        // Generate candidate and determine outcome (simplified simulation)
        let outcome = match selected {
            LeanStrategy::ForbiddenShortcut => {
                // Always rejected by Coh
                LeanOutcome::ForbiddenRejected
            }
            LeanStrategy::DirectGLB => {
                // Low success rate
                if rng.next_f64() < 0.05 {
                    LeanOutcome::LeanNearMiss
                } else {
                    LeanOutcome::LeanNearMiss
                }
            }
            LeanStrategy::PairwiseSumSet => {
                // Medium success
                if rng.next_f64() < 0.15 {
                    LeanOutcome::HelperReductionCompiled
                } else {
                    LeanOutcome::LeanNearMiss
                }
            }
            LeanStrategy::HelperDecomposition => {
                // Higher success with lower wildness
                if rng.next_f64() < (0.35 - wildness * 0.05).max(0.1) {
                    LeanOutcome::HelperReductionCompiled
                } else {
                    LeanOutcome::LeanNearMiss
                }
            }
            LeanStrategy::NamedMissingLemma => {
                // Good success
                if rng.next_f64() < 0.25 {
                    LeanOutcome::MissingLemmaIsolated
                } else {
                    LeanOutcome::LeanNearMiss
                }
            }
            LeanStrategy::LibrarySearch => {
                // Low-medium success
                if rng.next_f64() < 0.10 {
                    LeanOutcome::LeanNearMiss
                } else {
                    LeanOutcome::LeanNearMiss
                }
            }
        };

        outcomes.push((selected, outcome));
    }

    outcomes
}

/// Convert Lean outcome to BoundaryReceiptSummary
fn outcome_to_receipt(
    strategy: LeanStrategy,
    outcome: LeanOutcome,
    wildness: f64,
) -> BoundaryReceiptSummary {
    let accepted = matches!(
        outcome,
        LeanOutcome::HelperReductionCompiled | LeanOutcome::MissingLemmaIsolated
    );

    let first_failure = match outcome {
        LeanOutcome::FullOriginalProof => "none",
        LeanOutcome::HelperReductionCompiled => "none",
        LeanOutcome::MissingLemmaIsolated => "none_proved",
        LeanOutcome::LeanNearMiss => "lean_compile_error",
        LeanOutcome::ForbiddenRejected => "policy_violation",
    };

    let outcome_str = match outcome {
        LeanOutcome::FullOriginalProof => "accepted",
        LeanOutcome::HelperReductionCompiled => "accepted",
        LeanOutcome::MissingLemmaIsolated => "accepted",
        LeanOutcome::LeanNearMiss => "rejected",
        LeanOutcome::ForbiddenRejected => "rejected",
    };

    // Genesis margin: positive means good
    let genesis_margin = match outcome {
        LeanOutcome::FullOriginalProof => 100,
        LeanOutcome::HelperReductionCompiled => 80,
        LeanOutcome::MissingLemmaIsolated => 60,
        LeanOutcome::LeanNearMiss => -20,
        LeanOutcome::ForbiddenRejected => -100,
    };

    // Coherence margin: positive means good
    let coherence_margin = match outcome {
        LeanOutcome::FullOriginalProof => 100,
        LeanOutcome::HelperReductionCompiled => 70,
        LeanOutcome::MissingLemmaIsolated => 50,
        LeanOutcome::LeanNearMiss => -10,
        LeanOutcome::ForbiddenRejected => -80,
    };

    // Novelty: higher for more novel approaches
    let novelty = match outcome {
        LeanOutcome::FullOriginalProof => 0.0,
        LeanOutcome::HelperReductionCompiled => 0.3,
        LeanOutcome::MissingLemmaIsolated => 0.7,
        LeanOutcome::LeanNearMiss => 0.5,
        LeanOutcome::ForbiddenRejected => 0.1,
    };

    BoundaryReceiptSummary {
        domain: "lean_proof".to_string(),
        target: "isRationalInf_add_inf_le".to_string(),
        strategy_class: strategy.as_str().to_string(),
        wildness,
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
        ..Default::default()
    }
}

/// Calculate strategy entropy (Shannon entropy)
fn calculate_entropy(weights: &[(LeanStrategy, f64)]) -> f64 {
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
    println!("PhaseLoom-Guided Lean V3 Proof Engineering Loop");
    println!("============================================");
    println!();

    let config = PhaseLoomConfig {
        initial_budget: 10_000,
        learning_rate: 0.1,
        curvature_penalty: 0.05,
        circuit_break_threshold: 1000,
        min_weight: 0.01,
        ..Default::default()
    };

    // Initial uniform strategy weights
    let baseline_weights: Vec<(LeanStrategy, f64)> = vec![
        (LeanStrategy::DirectGLB, 1.0 / 6.0),
        (LeanStrategy::PairwiseSumSet, 1.0 / 6.0),
        (LeanStrategy::HelperDecomposition, 1.0 / 6.0),
        (LeanStrategy::NamedMissingLemma, 1.0 / 6.0),
        (LeanStrategy::LibrarySearch, 1.0 / 6.0),
        (LeanStrategy::ForbiddenShortcut, 1.0 / 6.0),
    ];

    let wildness = 1.5;

    // ========== PHASE 1: Baseline sweep ==========
    println!("--- Phase 1: Baseline Sweep (Uniform Weights) ---");

    let mut rng = SimpleRng::new(42);
    let baseline_outcomes = run_sweep(wildness, &baseline_weights, &mut rng);
    let initial_entropy = calculate_entropy(&baseline_weights);

    // Count outcomes
    let mut helper_compiled = 0;
    let mut missing_lemma = 0;
    let mut near_miss = 0;
    let mut forbidden = 0;

    for (_, outcome) in &baseline_outcomes {
        match outcome {
            LeanOutcome::HelperReductionCompiled => helper_compiled += 1,
            LeanOutcome::MissingLemmaIsolated => missing_lemma += 1,
            LeanOutcome::LeanNearMiss => near_miss += 1,
            LeanOutcome::ForbiddenRejected => forbidden += 1,
            LeanOutcome::FullOriginalProof => {}
        };
    }

    println!("Initial entropy: {:.3}", initial_entropy);
    println!("Helper compiled: {}", helper_compiled);
    println!("Missing lemma: {}", missing_lemma);
    println!("Near miss: {}", near_miss);
    println!("Forbidden rejected: {}", forbidden);
    println!();

    let baseline_helper = helper_compiled + missing_lemma;
    let baseline_forbidden = forbidden;

    // ========== PHASE 2: PhaseLoom adaptation ==========
    println!("--- Phase 2: PhaseLoom Adaptation ---");

    let mut state = phaseloom_init(&config);

    // Ingest all baseline outcomes
    for (strategy, outcome) in &baseline_outcomes {
        let receipt = outcome_to_receipt(*strategy, *outcome, wildness);
        phaseloom_ingest(&mut state, &receipt, &config);
    }

    println!("PhaseLoom state after ingestion:");
    println!("  tau: {}", state.tau);
    println!("  curvature: {}", state.curvature);
    println!("  budget: {}", state.budget);
    println!("  accepted: {}", state.accepted_count);
    println!("  rejected: {}", state.rejected_count);
    println!("  circuit broken: {}", state.circuit_broken);
    println!("  strategy weights: {:?}", state.all_weights());
    println!();

    // Build adapted weights from PhaseLoom state
    let mut adapted_weights: Vec<(LeanStrategy, f64)> = Vec::new();

    // Get weights for each strategy
    for strategy in &[
        LeanStrategy::DirectGLB,
        LeanStrategy::PairwiseSumSet,
        LeanStrategy::HelperDecomposition,
        LeanStrategy::NamedMissingLemma,
        LeanStrategy::LibrarySearch,
        LeanStrategy::ForbiddenShortcut,
    ] {
        let weight = state.weight_for(strategy.as_str());
        // Ensure minimum weight to prevent collapse
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

    let mut rng = SimpleRng::new(42); // Same seed for fair comparison
    let adapted_outcomes = run_sweep(wildness, &adapted_weights, &mut rng);

    // Count outcomes
    helper_compiled = 0;
    missing_lemma = 0;
    near_miss = 0;
    forbidden = 0;

    for (_, outcome) in &adapted_outcomes {
        match outcome {
            LeanOutcome::HelperReductionCompiled => helper_compiled += 1,
            LeanOutcome::MissingLemmaIsolated => missing_lemma += 1,
            LeanOutcome::LeanNearMiss => near_miss += 1,
            LeanOutcome::ForbiddenRejected => forbidden += 1,
            LeanOutcome::FullOriginalProof => {}
        };
    }

    println!("Helper compiled: {}", helper_compiled);
    println!("Missing lemma: {}", missing_lemma);
    println!("Near miss: {}", near_miss);
    println!("Forbidden rejected: {}", forbidden);
    println!();

    let adapted_helper = helper_compiled + missing_lemma;
    let adapted_forbidden = forbidden;

    // ========== RESULTS ==========
    println!("=== Results Summary ===");
    println!();
    println!("Metric                  Baseline    Adapted    Change");
    println!("------                  -------    -------    ------");

    let helper_change = if baseline_helper > 0 {
        ((adapted_helper as f64 - baseline_helper as f64) / baseline_helper as f64 * 100.0) as i32
    } else {
        0
    };
    println!(
        "Helper/MissLemma       {:>8}    {:>8}    {:>+5}%",
        baseline_helper, adapted_helper, helper_change
    );

    let forbidden_change = if baseline_forbidden > 0 {
        ((adapted_forbidden as f64 - baseline_forbidden as f64) / baseline_forbidden as f64 * 100.0)
            as i32
    } else {
        0
    };
    println!(
        "Forbidden rejected    {:>8}    {:>8}    {:>+5}%",
        baseline_forbidden, adapted_forbidden, forbidden_change
    );

    println!(
        "Strategy entropy       {:>8.3}    {:>8.3}",
        initial_entropy, adapted_entropy
    );

    println!();
    println!("Budget remaining: {}", state.budget);
    println!("Curvature: {}", state.curvature);
    println!(
        "Circuit broken: {}",
        phaseloom_circuit_broken(&state, &config)
    );

    // Success criteria
    println!();
    if adapted_helper >= baseline_helper && adapted_forbidden <= baseline_forbidden {
        println!("RESULT: PhaseLoom-guided sweep maintains or improves helper outcomes");
        println!("        while reducing forbidden shortcuts");
    } else {
        println!("RESULT: Needs tuning - check strategy weights");
    }

    println!();
    println!("PhaseLoom-Guided Lean V3 Proof Engineering Loop - Complete");
}
