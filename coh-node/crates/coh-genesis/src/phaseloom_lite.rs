//! PhaseLoomLite: Receipt-grounded adaptive memory for NPE strategy selection.
//!
//! PhaseLoom consumes boundary receipts from Coh verification and biases future NPE proposals.
//! **Safety Invariant**: PhaseLoom may bias proposals but never bypasses verification.
//!
//! ## State
//!
//! - `strategy_weights`: Learned weights for strategy classes (normalized to sum = 1.0)
//! - `curvature`: Accumulated rejection stress (increases on rejection)
//! - `budget`: Remaining thermodynamic work capacity
//! - `tau`: Intrinsic step counter
//!
//! ## Update Law
//!
//! ```
//! w_{c,n+1} = Normalize(w_{c,n} + η × R_c - ρ × F_c)
//! C_{n+1} = C_n + (rejected outcomes)
//! B_{n+1} = B_n - spend
//! ```

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for PhaseLoom initialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomConfig {
    /// Initial budget (thermodynamic work capacity)
    pub initial_budget: u128,
    /// Learning rate (positive reward magnitude)
    pub learning_rate: f64,
    /// Curvature penalty coefficient
    pub curvature_penalty: f64,
    /// Circuit breaker threshold (max curvature before pause)
    pub circuit_break_threshold: u128,
    /// Minimum weight value (prevent weight collapse)
    pub min_weight: f64,
    /// Initial temperature for Boltzmann exploration
    pub initial_temperature: f64,
    /// Decay rate for temperature per step
    pub temperature_decay: f64,
    /// Minimum temperature floor
    pub min_temperature: f64,
    /// Target entropy floor for exploration
    pub entropy_floor: f64,
    /// Max exploration failures before forcing exploitation
    pub exploration_failure_threshold: u64,
}

impl Default for PhaseLoomConfig {
    fn default() -> Self {
        Self {
            initial_budget: 100_000,
            learning_rate: 0.1,
            curvature_penalty: 0.05,
            circuit_break_threshold: 10_000,
            min_weight: 0.01,
            initial_temperature: 1.0,
            temperature_decay: 0.99,
            min_temperature: 0.1,
            entropy_floor: 0.5,
            exploration_failure_threshold: 5,
        }
    }
}

/// Strategy weight vector indexed by strategy_class
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyWeights(pub HashMap<String, f64>);

impl Default for StrategyWeights {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl StrategyWeights {
    /// Normalize weights to sum = 1.0 (probability distribution)
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.values().sum();
        if sum > 0.0 {
            for value in self.0.values_mut() {
                *value /= sum;
            }
        }
    }

    /// Get normalized weight for a class
    pub fn get(&self, class: &str) -> f64 {
        self.0.get(class).copied().unwrap_or(0.0)
    }

    /// Increment weight for a class by delta
    pub fn increment(&mut self, class: &str, delta: f64) {
        self.0
            .entry(class.to_string())
            .and_modify(|w| *w = (*w + delta).max(0.0))
            .or_insert(delta.max(0.0));
    }

    /// Get all strategy classes
    pub fn classes(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
}

/// PhaseLoom state: adaptive memory for NPE strategy selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomState {
    /// Strategy weight vector (bias toward proven useful outcomes)
    pub strategy_weights: StrategyWeights,
    /// Structural curvature (accumulated rejection stress)
    pub curvature: u128,
    /// Remaining budget (thermodynamic work capacity)
    pub budget: u128,
    /// Intrinsic machine time (step counter)
    pub tau: u64,
    /// Count of accepted outcomes
    pub accepted_count: u64,
    /// Count of rejected outcomes
    pub rejected_count: u64,
    /// Failure counts per strategy class
    pub failure_counts: HashMap<String, u64>,
    /// Circuit breaker triggered
    pub circuit_broken: bool,
    /// Current temperature for Boltzmann exploration (decays over tau)
    pub temperature: f64,
    /// Consecutive exploration failures (triggers exploitation mode)
    pub exploration_failure_count: u64,
    /// Current entropy of strategy distribution
    pub current_entropy: f64,
    /// Whether to force exploitation (overrides entropy floor)
    pub force_exploitation: bool,
}

impl Default for PhaseLoomState {
    fn default() -> Self {
        Self::new(&PhaseLoomConfig::default())
    }
}

impl PhaseLoomState {
    /// Create new PhaseLoomState with configuration
    pub fn new(config: &PhaseLoomConfig) -> Self {
        Self {
            strategy_weights: StrategyWeights::default(),
            curvature: 0,
            budget: config.initial_budget,
            tau: 0,
            accepted_count: 0,
            rejected_count: 0,
            failure_counts: HashMap::new(),
            circuit_broken: false,
            temperature: config.initial_temperature,
            exploration_failure_count: 0,
            current_entropy: 0.0,
            force_exploitation: false,
        }
    }

    /// Compute Shannon entropy of strategy distribution: H = -Σ p * log(p)
    pub fn compute_entropy(&self) -> f64 {
        let weights = &self.strategy_weights.0;
        if weights.is_empty() {
            return 0.0;
        }

        let sum: f64 = weights.values().sum();
        if sum == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for w in weights.values() {
            let p = w / sum;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        entropy
    }

    /// Update temperature with decay schedule
    pub fn update_temperature(&mut self, config: &PhaseLoomConfig) {
        // Decay temperature, floor at min_temperature
        self.temperature =
            (self.temperature * config.temperature_decay).max(config.min_temperature);
    }

    /// Check if entropy floor is satisfied
    pub fn satisfies_entropy_floor(&self, config: &PhaseLoomConfig) -> bool {
        self.current_entropy >= config.entropy_floor || self.strategy_weights.0.len() < 2
    }

    /// Sample a strategy using Boltzmann exploration with entropy floor
    /// Returns (strategy_name, was_exploration)
    pub fn sample_strategy<R: rand::Rng>(
        &self,
        config: &PhaseLoomConfig,
        rng: &mut R,
    ) -> (Option<String>, bool) {
        // Force exploitation if circuit broken or too many exploration failures
        if self.force_exploitation || self.circuit_broken {
            // Exploitation: pick highest weight
            let best = self
                .strategy_weights
                .0
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| k.clone());
            return (best, false);
        }

        let weights = &self.strategy_weights.0;
        if weights.is_empty() {
            return (None, false);
        }

        // Check entropy floor - if violated, force exploration
        let should_explore = !self.satisfies_entropy_floor(config)
            || self.exploration_failure_count >= config.exploration_failure_threshold;

        if should_explore || self.temperature > config.min_temperature {
            return (self.sample_boltzmann(rng), true);
        }

        (self.sample_best(), false)
    }

    /// Sample strategy using Boltzmann exploration
    pub fn sample_boltzmann<R: rand::Rng>(&self, rng: &mut R) -> Option<String> {
        let weights = &self.strategy_weights.0;
        let mut sum = 0.0;
        let mut exp_weights = Vec::new();
        for (class, weight) in weights {
            let exp_w = (weight / self.temperature.max(0.001)).exp();
            sum += exp_w;
            exp_weights.push((class, exp_w));
        }

        if sum > 0.0 {
            let r = rng.gen_range(0.0..sum);
            let mut acc = 0.0;
            for (class, exp_w) in exp_weights {
                acc += exp_w;
                if r <= acc {
                    return Some(class.clone());
                }
            }
        }

        self.sample_best()
    }

    /// Sample strategy with highest weight (exploitation only)
    pub fn sample_best(&self) -> Option<String> {
        if self.strategy_weights.0.is_empty() {
            return None;
        }

        self.strategy_weights
            .0
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
    }

    /// Check circuit breaker - pause learning if curvature too high or budget exhausted
    pub fn is_circuit_broken(&self, config: &PhaseLoomConfig) -> bool {
        self.curvature > config.circuit_break_threshold || self.budget == 0
    }

    /// Process a boundary receipt and update internal state
    /// This implements the update law from the integration plan
    pub fn ingest(&mut self, receipt: &BoundaryReceiptSummary, config: &PhaseLoomConfig) {
        // Update step counter (intrinsic time)
        self.tau = self.tau.saturating_add(1);

        // Curvature accumulation (rejection penalty)
        if !receipt.accepted {
            self.curvature = self.curvature.saturating_add(1);
            self.rejected_count = self.rejected_count.saturating_add(1);

            // Track failure count per strategy class
            self.failure_counts
                .entry(receipt.strategy_class.clone())
                .and_modify(|c| *c = c.saturating_add(1))
                .or_insert(1);
        } else {
            self.accepted_count = self.accepted_count.saturating_add(1);
        }

        // Budget burn (work consumption)
        let spend = if receipt.accepted { 10 } else { 50 };
        self.budget = self.budget.saturating_sub(spend);

        // Strategy weight update: reward acceptance, penalize failure
        let class = &receipt.strategy_class;
        let reward = if receipt.accepted {
            config.learning_rate * receipt.novelty.max(0.1)
        } else {
            0.0
        };
        let penalty = if !receipt.accepted {
            let failure_count = self.failure_counts.get(class).copied().unwrap_or(1) as f64;
            config.curvature_penalty * failure_count.min(0.5)
        } else {
            0.0
        };

        self.strategy_weights.increment(class, reward - penalty);
        self.strategy_weights.normalize();

        // Check circuit breaker
        self.circuit_broken = self.is_circuit_broken(config);
    }

    /// Get weight for a specific strategy class
    pub fn weight_for(&self, class: &str) -> f64 {
        self.strategy_weights.get(class)
    }

    /// Get all strategy classes with weights
    pub fn all_weights(&self) -> &HashMap<String, f64> {
        &self.strategy_weights.0
    }

    /// Serialize state to JSON bytes
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize state from JSON bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Boundary receipt summary consumed by PhaseLoom
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryReceiptSummary {
    /// Domain (e.g., "code", "test", "docs")
    pub domain: String,
    /// Target (e.g., "function foo", "module bar")
    pub target: String,
    /// Strategy class used (e.g., "synthesize", "refine", "debug")
    pub strategy_class: String,
    /// Wildness parameter (0.0 = conservative, 1.0 = aggressive)
    pub wildness: f64,
    /// Genesis margin: M(g') + C(p) - M(g) - D(p)
    pub genesis_margin: i128,
    /// Coherence margin: V_post + spend - V_pre - defect
    pub coherence_margin: i128,
    /// First failure reason if rejected
    pub first_failure: String,
    /// Outcome: "accepted", "rejected", "erroneous"
    pub outcome: String,
    /// Accepted: true/false
    pub accepted: bool,
    /// Novelty score (0.0 = repeat, 1.0 = novel)
    pub novelty: f64,
    /// Receipt hash for audit trail
    pub receipt_hash: String,
    // ==== Mathlib fields for PhaseLoom learning ====
    /// Mathlib strategy used (e.g., "IsGLB", "SInf", "OrderTheory")
    pub mathlib_strategy: Option<String>,
    /// Mathlib confidence (0.0 - 1.0)
    pub mathlib_confidence: Option<f64>,
    /// Suggested lemmas from mathlib
    pub mathlib_suggested_lemmas: Option<Vec<String>>,
    /// Mathlib import risk tier
    pub mathlib_import_risk: Option<String>,
    /// Were imports actually used in proof?
    pub mathlib_imports_used: bool,
    /// What mathlib effect was observed
    pub mathlib_effect: MathlibEffect,
    // ==== Failure Taxonomy fields ====
    /// Failure layer (if rejected)
    pub failure_layer: Option<String>,
    /// Detailed failure kind (if rejected)
    pub failure_kind: Option<String>,
}

/// Effect of mathlib on the proof attempt
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MathlibEffect {
    /// No mathlib used
    #[default]
    None,
    /// Strategy guidance only, no imports
    StrategyOnly,
    /// Imports helped the proof
    ImportHelped,
    /// Policy rejected the imports
    ImportRejected,
    /// Imports neither helped nor hurt
    ImportNeutral,
}

impl Default for BoundaryReceiptSummary {
    fn default() -> Self {
        Self {
            domain: String::new(),
            target: String::new(),
            strategy_class: String::new(),
            wildness: 0.0,
            genesis_margin: 0,
            coherence_margin: 0,
            first_failure: String::new(),
            outcome: String::new(),
            accepted: false,
            novelty: 0.0,
            receipt_hash: String::new(),
            mathlib_strategy: None,
            mathlib_confidence: None,
            mathlib_suggested_lemmas: None,
            mathlib_import_risk: None,
            mathlib_imports_used: false,
            mathlib_effect: MathlibEffect::None,
            failure_layer: None,
            failure_kind: None,
        }
    }
}

impl BoundaryReceiptSummary {
    /// Create a test receipt for unit testing
    #[cfg(test)]
    pub fn test_accepted(strategy_class: &str, novelty: f64) -> Self {
        Self {
            domain: "test".to_string(),
            target: "test_fn".to_string(),
            strategy_class: strategy_class.to_string(),
            wildness: 0.5,
            genesis_margin: 100,
            coherence_margin: 50,
            first_failure: String::new(),
            outcome: "accepted".to_string(),
            accepted: true,
            novelty,
            receipt_hash: "test_hash".to_string(),
            mathlib_strategy: None,
            mathlib_confidence: None,
            mathlib_suggested_lemmas: None,
            mathlib_import_risk: None,
            mathlib_imports_used: false,
            mathlib_effect: MathlibEffect::None,
            failure_layer: None,
            failure_kind: None,
        }
    }

    #[cfg(test)]
    pub fn test_rejected(strategy_class: &str, failure: &str) -> Self {
        Self {
            domain: "test".to_string(),
            target: "test_fn".to_string(),
            strategy_class: strategy_class.to_string(),
            wildness: 0.5,
            genesis_margin: -10,
            coherence_margin: -20,
            first_failure: failure.to_string(),
            outcome: "rejected".to_string(),
            accepted: false,
            novelty: 0.0,
            receipt_hash: "test_hash".to_string(),
            mathlib_strategy: None,
            mathlib_confidence: None,
            mathlib_suggested_lemmas: None,
            mathlib_import_risk: None,
            mathlib_imports_used: false,
            mathlib_effect: MathlibEffect::None,
            failure_layer: Some("Genesis".to_string()),
            failure_kind: Some("NegativeMargin".to_string()),
        }
    }
}

/// Public API functions

/// Initialize PhaseLoom state
pub fn phaseloom_init(config: &PhaseLoomConfig) -> PhaseLoomState {
    PhaseLoomState::new(config)
}

/// Ingest a boundary receipt
pub fn phaseloom_ingest(
    state: &mut PhaseLoomState,
    receipt: &BoundaryReceiptSummary,
    config: &PhaseLoomConfig,
) {
    state.ingest(receipt, config);
}

/// Sample next strategy (exploitation only)
pub fn phaseloom_sample(state: &PhaseLoomState) -> Option<String> {
    state.sample_best()
}

/// Sample next strategy using Boltzmann exploration
pub fn phaseloom_sample_boltzmann<R: rand::Rng>(
    state: &PhaseLoomState,
    config: &PhaseLoomConfig,
    rng: &mut R,
) -> (Option<String>, bool) {
    state.sample_strategy(config, rng)
}

/// Check circuit breaker
pub fn phaseloom_circuit_broken(state: &PhaseLoomState, config: &PhaseLoomConfig) -> bool {
    state.is_circuit_broken(config)
}

/// Serialize state for persistence
pub fn phaseloom_serialize(state: &PhaseLoomState) -> Result<Vec<u8>, serde_json::Error> {
    state.serialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phaseloom_init() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        assert_eq!(state.curvature, 0);
        assert_eq!(state.budget, config.initial_budget);
        assert_eq!(state.tau, 0);
        assert!(!state.circuit_broken);
    }

    #[test]
    fn test_ingest_accepted() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let receipt = BoundaryReceiptSummary::test_accepted("synthesize", 0.8);
        state.ingest(&receipt, &config);

        assert_eq!(state.tau, 1);
        assert_eq!(state.accepted_count, 1);
        assert_eq!(state.curvature, 0); // No curvature increase on accept
        assert!(state.strategy_weights.0.contains_key("synthesize"));
    }

    #[test]
    fn test_ingest_rejected() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let receipt = BoundaryReceiptSummary::test_rejected("synthesize", "policy_violation");
        state.ingest(&receipt, &config);

        assert_eq!(state.tau, 1);
        assert_eq!(state.rejected_count, 1);
        assert_eq!(state.curvature, 1); // Curvature increases on reject
    }

    #[test]
    fn test_weight_normalization() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Ingest multiple receipts
        state.ingest(
            &BoundaryReceiptSummary::test_accepted("synthesize", 0.5),
            &config,
        );
        state.ingest(
            &BoundaryReceiptSummary::test_accepted("refine", 0.5),
            &config,
        );
        state.ingest(
            &BoundaryReceiptSummary::test_rejected("debug", "error"),
            &config,
        );

        let sum: f64 = state.strategy_weights.0.values().sum();
        assert!((sum - 1.0).abs() < 0.001, "Weights should normalize to 1.0");
    }

    #[test]
    fn test_circuit_break() {
        let config = PhaseLoomConfig {
            circuit_break_threshold: 3,
            ..Default::default()
        };
        let mut state = PhaseLoomState::new(&config);

        // Reject enough to trigger circuit break (curvature = 4 > 3)
        for _ in 0..4 {
            state.ingest(
                &BoundaryReceiptSummary::test_rejected("test", "error"),
                &config,
            );
        }

        assert!(state.circuit_broken);
    }

    #[test]
    fn test_sample_strategy() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Add multiple strategies to have meaningful entropy
        state
            .strategy_weights
            .0
            .insert("synthesize".to_string(), 0.6);
        state.strategy_weights.0.insert("refine".to_string(), 0.3);
        state.strategy_weights.0.insert("debug".to_string(), 0.1);
        state.strategy_weights.normalize();

        let mut rng = rand::thread_rng();
        let (strategy, was_exploration) = state.sample_strategy(&config, &mut rng);

        // Should get a valid strategy
        assert!(strategy.is_some());
        // Strategy should be one of the ones we added
        assert!(matches!(
            strategy.as_deref(),
            Some("synthesize") | Some("refine") | Some("debug")
        ));
    }

    #[test]
    fn test_compute_entropy() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        // Empty weights = zero entropy
        let entropy = state.compute_entropy();
        assert!((entropy - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_tracking() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        // Initially entropy should be 0
        assert!((state.current_entropy - 0.0).abs() < 0.001);

        // After adding a strategy, entropy gets recomputed on ingest
        state
            .strategy_weights
            .0
            .insert("synthesize".to_string(), 1.0);
        state.strategy_weights.normalize();
        state.current_entropy = state.compute_entropy();

        // Single strategy = low entropy
        let entropy = state.current_entropy;
        assert!(entropy < 0.5, "Single strategy should have low entropy");
    }

    #[test]
    fn test_temperature_decay() {
        let config = PhaseLoomConfig::default();
        let mut state = PhaseLoomState::new(&config);

        let initial_temp = state.temperature;

        // Decay multiple times
        for _ in 0..10 {
            state.update_temperature(&config);
        }

        // Temperature should have decayed
        assert!(state.temperature < initial_temp);

        // But should not go below min_temperature
        assert!(state.temperature >= config.min_temperature);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = PhaseLoomConfig::default();
        let state = PhaseLoomState::new(&config);

        let bytes = state.serialize().unwrap();
        let restored = PhaseLoomState::deserialize(&bytes).unwrap();

        assert_eq!(state.tau, restored.tau);
        assert_eq!(state.budget, restored.budget);
    }
}
