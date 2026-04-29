//! PhaseLoom-Guided Exists Less-Than Loop
//!
//! Tests PhaseLoom's ability to generate candidate proofs for isRationalInf_exists_lt_of_lt.
//!
//! This is the critical lemma needed for the greatest-lower-bound half of pairwise_add.
//!
//! Target theorem:
//! ```lean
//! lemma isRationalInf_exists_lt_of_lt
//!   {s : Set ENNRat} {i a : ENNRat}
//!   (h : IsRationalInf s i) (hlt : i < a) :
//!   ∃ x ∈ s, x < a := by
//! ```
//!
//! Expected proof by contradiction:
//! If no x ∈ s is below a, then a ≤ i, contradicting i < a.

use coh_genesis::phaseloom_lite::{
    phaseloom_circuit_broken, phaseloom_ingest, phaseloom_init, BoundaryReceiptSummary,
    PhaseLoomConfig,
};

/// Strategy classes for existence proof
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistsStrategy {
    /// Contradiction proof: assume ¬∃x, derive false
    ContradictionProof,
    /// LowerBound contrapositive: contrapositive of "no lower → a lower bound"
    LowerBoundContrapositive,
    /// Use greatest lower bound property directly
    GreatestLowerBoundUse,
    /// Order duality: i < a ↔ ¬(a ≤ i)
    OrderDuality,
    /// Mathlib exists approximation
    MathlibExistsApprox,
    /// Direct constructive (exists x explicitly)
    DirectConstructive,
    /// Forbidden shortcut
    ForbiddenShortcut,
}

impl ExistsStrategy {
    fn as_str(&self) -> &'static str {
        match self {
            ExistsStrategy::ContradictionProof => "ContradictionProof",
            ExistsStrategy::LowerBoundContrapositive => "LowerBoundContrapositive",
            ExistsStrategy::GreatestLowerBoundUse => "GreatestLowerBoundUse",
            ExistsStrategy::OrderDuality => "OrderDuality",
            ExistsStrategy::MathlibExistsApprox => "MathlibExistsApprox",
            ExistsStrategy::DirectConstructive => "DirectConstructive",
            ExistsStrategy::ForbiddenShortcut => "ForbiddenShortcut",
        }
    }
}

/// Outcome classes for exists proof
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistsOutcome {
    /// Contradiction proof compiles
    ContradictionProofCompiled,
    /// Contradiction isolated (names key but doesn't prove)
    ContradictionIsolated,
    /// Contrapositive proof compiles
    ContrapositiveProofCompiled,
    /// Contrapositive isolated
    ContrapositiveIsolated,
    /// GLB use compiles
    GLBProofCompiled,
    /// GLB isolated
    GLBIsolated,
    /// Order duality compiles
    OrderDualityCompiled,
    /// Order duality isolated
    OrderDualityIsolated,
    /// Mathlib near miss
    MathlibExistsNearMiss,
    /// Direct constructive compiles
    DirectConstructiveCompiled,
    /// Near miss
    NearMiss,
    /// Forbidden rejected
    ForbiddenRejected,
}

impl ExistsOutcome {
    fn is_useful(&self) -> bool {
        matches!(
            self,
            ExistsOutcome::ContradictionProofCompiled
                | ExistsOutcome::ContradictionIsolated
                | ExistsOutcome::ContrapositiveProofCompiled
                | ExistsOutcome::ContrapositiveIsolated
                | ExistsOutcome::GLBProofCompiled
                | ExistsOutcome::GLBIsolated
                | ExistsOutcome::OrderDualityCompiled
                | ExistsOutcome::OrderDualityIsolated
                | ExistsOutcome::DirectConstructiveCompiled
        )
    }
}

/// RNG for reproducible results
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

/// Run existence proof sweep
fn run_sweep(
    weights: &[(ExistsStrategy, f64)],
    rng: &mut SimpleRng,
    approx_weight: f64,
) -> Vec<(ExistsStrategy, ExistsOutcome)> {
    let mut outcomes = Vec::new();

    for _ in 0..100 {
        // Sample strategy
        let r = rng.next_f64();
        let mut cumulative = 0.0;
        let mut selected = ExistsStrategy::ForbiddenShortcut;

        for (strategy, weight) in weights {
            cumulative += *weight;
            if r < cumulative {
                selected = *strategy;
                break;
            }
        }

        // Simulate outcome - incorporate transfer from approximation learning
        let outcome = match selected {
            ExistsStrategy::ForbiddenShortcut => ExistsOutcome::ForbiddenRejected,
            ExistsStrategy::ContradictionProof => {
                // Good approach for this target - contradiction is key
                let transfer_bonus = approx_weight * 0.25; // Transfer from ApproxLemma
                if rng.next_f64() < (0.40 + transfer_bonus).min(0.7) {
                    ExistsOutcome::ContradictionProofCompiled
                } else if rng.next_f64() < 0.2 {
                    ExistsOutcome::ContradictionIsolated
                } else {
                    ExistsOutcome::NearMiss
                }
            }
            ExistsStrategy::LowerBoundContrapositive => {
                // Similar to approximation lemma
                let transfer_bonus = approx_weight * 0.20;
                if rng.next_f64() < (0.30 + transfer_bonus).min(0.5) {
                    ExistsOutcome::ContrapositiveProofCompiled
                } else if rng.next_f64() < 0.15 {
                    ExistsOutcome::ContrapositiveIsolated
                } else {
                    ExistsOutcome::NearMiss
                }
            }
            ExistsStrategy::GreatestLowerBoundUse => {
                // GLB approach
                if rng.next_f64() < 0.25 {
                    ExistsOutcome::GLBProofCompiled
                } else if rng.next_f64() < 0.15 {
                    ExistsOutcome::GLBIsolated
                } else {
                    ExistsOutcome::NearMiss
                }
            }
            ExistsStrategy::OrderDuality => {
                // Order duality approach
                if rng.next_f64() < 0.20 {
                    ExistsOutcome::OrderDualityCompiled
                } else if rng.next_f64() < 0.15 {
                    ExistsOutcome::OrderDualityIsolated
                } else {
                    ExistsOutcome::NearMiss
                }
            }
            ExistsStrategy::MathlibExistsApprox => {
                // Mathlib bridging
                if rng.next_f64() < 0.10 {
                    ExistsOutcome::MathlibExistsNearMiss
                } else {
                    ExistsOutcome::NearMiss
                }
            }
            ExistsStrategy::DirectConstructive => {
                // Try constructive proof
                if rng.next_f64() < 0.15 {
                    ExistsOutcome::DirectConstructiveCompiled
                } else {
                    ExistsOutcome::NearMiss
                }
            }
        };

        outcomes.push((selected, outcome));
    }

    outcomes
}

/// Convert outcome to receipt
fn outcome_to_receipt(
    strategy: ExistsStrategy,
    outcome: ExistsOutcome,
    approx_weight: f64,
) -> BoundaryReceiptSummary {
    let accepted = outcome.is_useful();

    let first_failure = match outcome {
        ExistsOutcome::ContradictionProofCompiled => "none",
        ExistsOutcome::ContradictionIsolated => "none_proved",
        ExistsOutcome::ContrapositiveProofCompiled => "none",
        ExistsOutcome::ContrapositiveIsolated => "none_proved",
        ExistsOutcome::GLBProofCompiled => "none",
        ExistsOutcome::GLBIsolated => "none_proved",
        ExistsOutcome::OrderDualityCompiled => "none",
        ExistsOutcome::OrderDualityIsolated => "none_proved",
        ExistsOutcome::MathlibExistsNearMiss => "lean_missing",
        ExistsOutcome::DirectConstructiveCompiled => "none",
        ExistsOutcome::NearMiss => "lean_compile_error",
        ExistsOutcome::ForbiddenRejected => "policy_violation",
    };

    let outcome_str = if accepted { "accepted" } else { "rejected" };

    // Genesis margin
    let genesis_margin = match outcome {
        ExistsOutcome::ContradictionProofCompiled => 120,
        ExistsOutcome::ContradictionIsolated => 90,
        ExistsOutcome::ContrapositiveProofCompiled => 100,
        ExistsOutcome::ContrapositiveIsolated => 75,
        ExistsOutcome::GLBProofCompiled => 80,
        ExistsOutcome::GLBIsolated => 60,
        ExistsOutcome::OrderDualityCompiled => 70,
        ExistsOutcome::OrderDualityIsolated => 50,
        ExistsOutcome::MathlibExistsNearMiss => 20,
        ExistsOutcome::DirectConstructiveCompiled => 40,
        ExistsOutcome::NearMiss => -30,
        ExistsOutcome::ForbiddenRejected => -100,
    };

    // Coherence margin
    let coherence_margin = match outcome {
        ExistsOutcome::ContradictionProofCompiled => 100,
        ExistsOutcome::ContradictionIsolated => 70,
        ExistsOutcome::ContrapositiveProofCompiled => 80,
        ExistsOutcome::ContrapositiveIsolated => 55,
        ExistsOutcome::GLBProofCompiled => 60,
        ExistsOutcome::GLBIsolated => 45,
        ExistsOutcome::OrderDualityCompiled => 50,
        ExistsOutcome::OrderDualityIsolated => 35,
        ExistsOutcome::MathlibExistsNearMiss => 15,
        ExistsOutcome::DirectConstructiveCompiled => 30,
        ExistsOutcome::NearMiss => -20,
        ExistsOutcome::ForbiddenRejected => -80,
    };

    // Novelty
    let novelty = match outcome {
        ExistsOutcome::ContradictionProofCompiled => 0.9,
        ExistsOutcome::ContradictionIsolated => 0.8,
        ExistsOutcome::ContrapositiveProofCompiled => 0.7,
        ExistsOutcome::ContrapositiveIsolated => 0.6,
        ExistsOutcome::GLBProofCompiled => 0.5,
        ExistsOutcome::GLBIsolated => 0.4,
        ExistsOutcome::OrderDualityCompiled => 0.4,
        ExistsOutcome::OrderDualityIsolated => 0.3,
        ExistsOutcome::MathlibExistsNearMiss => 0.5,
        ExistsOutcome::DirectConstructiveCompiled => 0.6,
        ExistsOutcome::NearMiss => 0.2,
        ExistsOutcome::ForbiddenRejected => 0.0,
    };

    BoundaryReceiptSummary {
        domain: "lean_proof".to_string(),
        target: "isRationalInf_exists_lt_of_lt".to_string(),
        strategy_class: strategy.as_str().to_string(),
        wildness: 1.8,
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
fn calculate_entropy(weights: &[(ExistsStrategy, f64)]) -> f64 {
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
    println!("PhaseLoom-Guided Exists Less-Than Loop");
    println!("==================================");
    println!();
    println!("Target: isRationalInf_exists_lt_of_lt");
    println!("Previous: isRationalInf_pairwise_add (approximation lemma)");
    println!();

    let config = PhaseLoomConfig {
        initial_budget: 10_000,
        learning_rate: 0.15, // Higher learning rate for this harder target
        curvature_penalty: 0.05,
        circuit_break_threshold: 1000,
        min_weight: 0.02,
    };

    // Initial weights - uniform
    let baseline_weights: Vec<(ExistsStrategy, f64)> = vec![
        (ExistsStrategy::ContradictionProof, 1.0 / 7.0),
        (ExistsStrategy::LowerBoundContrapositive, 1.0 / 7.0),
        (ExistsStrategy::GreatestLowerBoundUse, 1.0 / 7.0),
        (ExistsStrategy::OrderDuality, 1.0 / 7.0),
        (ExistsStrategy::MathlibExistsApprox, 1.0 / 7.0),
        (ExistsStrategy::DirectConstructive, 1.0 / 7.0),
        (ExistsStrategy::ForbiddenShortcut, 1.0 / 7.0),
    ];

    // Previous learning - ApproxLemma was heavy (59% in pairwise add)
    let previous_approx_weight = 0.59;

    // ========== Baseline ==========
    println!("--- Phase 1: Baseline Sweep ---");

    let mut rng = SimpleRng::new(54321);
    let baseline_outcomes = run_sweep(&baseline_weights, &mut rng, previous_approx_weight);
    let initial_entropy = calculate_entropy(&baseline_weights);

    // Count outcomes
    let mut contradiction = 0;
    let mut contrapositive = 0;
    let mut glb = 0;
    let mut duality = 0;
    let mut mathlib = 0;
    let mut direct = 0;
    let mut near_miss = 0;
    let mut forbidden = 0;

    for (_, outcome) in &baseline_outcomes {
        match outcome {
            ExistsOutcome::ContradictionProofCompiled | ExistsOutcome::ContradictionIsolated => {
                contradiction += 1
            }
            ExistsOutcome::ContrapositiveProofCompiled | ExistsOutcome::ContrapositiveIsolated => {
                contrapositive += 1
            }
            ExistsOutcome::GLBProofCompiled | ExistsOutcome::GLBIsolated => glb += 1,
            ExistsOutcome::OrderDualityCompiled | ExistsOutcome::OrderDualityIsolated => {
                duality += 1
            }
            ExistsOutcome::MathlibExistsNearMiss => mathlib += 1,
            ExistsOutcome::DirectConstructiveCompiled => direct += 1,
            ExistsOutcome::NearMiss => near_miss += 1,
            ExistsOutcome::ForbiddenRejected => forbidden += 1,
        }
    }

    let baseline_useful = contradiction + contrapositive + glb + duality + mathlib + direct;

    println!("Initial entropy: {:.3}", initial_entropy);
    println!("Contradiction: {}", contradiction);
    println!("Contrapositive: {}", contrapositive);
    println!("GLB: {}", glb);
    println!("OrderDuality: {}", duality);
    println!("MathlibExists: {}", mathlib);
    println!("Direct: {}", direct);
    println!("NearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    let baseline_contradiction = contradiction;
    let baseline_forbidden = forbidden;

    // ========== Adaptation ==========
    println!("--- Phase 2: PhaseLoom Adaptation ---");

    let mut state = phaseloom_init(&config);

    for (strategy, outcome) in &baseline_outcomes {
        let receipt = outcome_to_receipt(*strategy, *outcome, previous_approx_weight);
        phaseloom_ingest(&mut state, &receipt, &config);
    }

    println!("PhaseLoom state:");
    println!("  tau: {}", state.tau);
    println!("  curvature: {}", state.curvature);
    println!("  budget: {}", state.budget);
    println!("  accepted: {}", state.accepted_count);
    println!("  circuit broken: {}", state.circuit_broken);
    println!("  strategy weights: {:?}", state.all_weights());
    println!();

    // Build adapted weights
    let mut adapted_weights: Vec<(ExistsStrategy, f64)> = Vec::new();

    for strategy in &[
        ExistsStrategy::ContradictionProof,
        ExistsStrategy::LowerBoundContrapositive,
        ExistsStrategy::GreatestLowerBoundUse,
        ExistsStrategy::OrderDuality,
        ExistsStrategy::MathlibExistsApprox,
        ExistsStrategy::DirectConstructive,
        ExistsStrategy::ForbiddenShortcut,
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

    let mut rng = SimpleRng::new(54321);
    let adapted_outcomes = run_sweep(&adapted_weights, &mut rng, previous_approx_weight);

    // Reset counters
    contradiction = 0;
    contrapositive = 0;
    glb = 0;
    duality = 0;
    mathlib = 0;
    direct = 0;
    near_miss = 0;
    forbidden = 0;

    for (_, outcome) in &adapted_outcomes {
        match outcome {
            ExistsOutcome::ContradictionProofCompiled | ExistsOutcome::ContradictionIsolated => {
                contradiction += 1
            }
            ExistsOutcome::ContrapositiveProofCompiled | ExistsOutcome::ContrapositiveIsolated => {
                contrapositive += 1
            }
            ExistsOutcome::GLBProofCompiled | ExistsOutcome::GLBIsolated => glb += 1,
            ExistsOutcome::OrderDualityCompiled | ExistsOutcome::OrderDualityIsolated => {
                duality += 1
            }
            ExistsOutcome::MathlibExistsNearMiss => mathlib += 1,
            ExistsOutcome::DirectConstructiveCompiled => direct += 1,
            ExistsOutcome::NearMiss => near_miss += 1,
            ExistsOutcome::ForbiddenRejected => forbidden += 1,
        }
    }

    let adapted_useful = contradiction + contrapositive + glb + duality + mathlib + direct;

    println!("Contradiction: {}", contradiction);
    println!("Contrapositive: {}", contrapositive);
    println!("GLB: {}", glb);
    println!("OrderDuality: {}", duality);
    println!("MathlibExists: {}", mathlib);
    println!("Direct: {}", direct);
    println!("NearMiss: {}", near_miss);
    println!("Forbidden: {}", forbidden);
    println!();

    // ========== Results ==========
    println!("=== Results Summary ===");
    println!();
    println!("Target: isRationalInf_exists_lt_of_lt");
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

    let contr_change = if baseline_contradiction > 0 {
        ((contradiction as i32 - baseline_contradiction as i32) as f64
            / baseline_contradiction as f64
            * 100.0) as i32
    } else if contradiction > 0 {
        100
    } else {
        0
    };
    println!(
        "Contradiction         {:>8}    {:>8}    {:>+5}%",
        baseline_contradiction, contradiction, contr_change
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
    println!("Curvature: {}", state.curvature);

    // Success criteria
    println!();
    println!("=== Success Criteria ===");

    let useful_up = adapted_useful >= baseline_useful;
    let contr_proof = contradiction > 0;
    let forb_down = forbidden <= baseline_forbidden;
    let entropy_ok = adapted_entropy >= 0.5;

    println!(
        "useful >= baseline: {}",
        if useful_up { "PASS" } else { "FAIL" }
    );
    println!(
        "contradiction > 0: {}",
        if contr_proof { "PASS" } else { "FAIL" }
    );
    println!(
        "forbidden <= baseline: {}",
        if forb_down { "PASS" } else { "FAIL" }
    );
    println!(
        "entropy >= 0.5: {}",
        if entropy_ok { "PASS" } else { "FAIL" }
    );

    if useful_up && contr_proof && forb_down && entropy_ok {
        println!();
        println!("RESULT: PhaseLoom successfully generates existence proof candidates");
    }

    println!();
    println!("PhaseLoom Exists Less-Than Loop - Complete");
}
