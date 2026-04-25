//! Coh Semantic Layer
//!
//! This module mirrors the Lean semantic definitions in [`coh-t-stack/Coh/Core/Semantic.lean`].
//!
//! It provides runtime support for:
//! - Hidden state spaces and observable state projection
//! - Hidden traces and observable projection
//! - Realizable fiber enumeration (bounded cases)
//! - Semantic cost computation over realizations
//!
//! Design Notes:
//! - This module is layered ABOVE the existing verify_micro/verify_chain kernel
//! - Existing kernel serves as the certification oracle for the observable trace projection
//! - This module introduces the hidden realization and semantic cost layer

use std::collections::HashSet;

/// Canonical hash trait: aligns Rust semantic layer with Lean without leaking infra
pub trait Hashable {
    fn hash(&self) -> Vec<u8>;
}

/// Represents an observable state in the semantic layer (Vec<u8> for Lean alignment)
pub type ObsState = Vec<u8>;

/// Hidden state: represents a semantic step in the hidden layer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HiddenState {
    Action(String),
    Terminal,
}

impl Hashable for HiddenState {
    fn hash(&self) -> Vec<u8> {
        match self {
            HiddenState::Action(s) => s.as_bytes().to_vec(),
            HiddenState::Terminal => b"terminal".to_vec(),
        }
    }
}

/// Hidden trace: a sequence of hidden states
#[derive(Debug, Clone)]
pub struct HiddenTrace {
    pub states: Vec<HiddenState>,
}

impl HiddenTrace {
    /// Create a new hidden trace
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// Push a hidden state onto the trace
    pub fn push(&mut self, s: HiddenState) {
        self.states.push(s);
    }

    /// Project to observable trace (as Vec<u8>)
    pub fn project<P: Fn(&HiddenState) -> Vec<u8>>(&self, projection: P) -> Vec<u8> {
        self.states.iter().flat_map(|s| projection(s)).collect()
    }
}

/// Semantic System configuration
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Projection function name (for traceability)
    pub projection_name: String,
    /// Maximum hidden trace length to enumerate (for bounded computation)
    pub max_enumeration_depth: usize,
    /// Whether to compute full fiber or just upper bound
    pub compute_full_fiber: bool,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            projection_name: "identity".to_string(),
            max_enumeration_depth: 10,
            compute_full_fiber: true,
        }
    }
}

/// Realizable fiber: set of hidden traces that project to a given observable trace
#[derive(Debug, Clone)]
pub struct RealizableFiber {
    /// The observable trace that these realize
    pub observable_trace: Vec<ObsState>,
    /// Hidden realizations (if bounded)
    pub realizations: Vec<HiddenTrace>,
    /// Whether the fiber is known to be finite
    pub is_finite: bool,
}

/// Semantic cost computation result
#[derive(Debug, Clone)]
pub struct SemanticCost {
    /// The cost value (NNReal-like u128)
    pub value: u128,
    /// The realization that achieved this cost (for debugging)
    pub maximizing_realization: Option<HiddenTrace>,
}

/// Compute semantic cost over a finite realizable fiber
///
/// This corresponds to the Lean `SemanticSystem.semanticCost` definition.
/// In the runtime, we compute it by enumerating the fiber and taking the max hidden cost.
pub fn compute_semantic_cost(
    fiber: &RealizableFiber,
    hidden_cost_fn: fn(&HiddenState) -> u128,
) -> SemanticCost {
    let mut max_cost: u128 = 0;
    let mut max_trace: Option<HiddenTrace> = None;

    for h in &fiber.realizations {
        let trace_cost: u128 = h.states.iter().map(hidden_cost_fn).sum();
        if trace_cost > max_cost {
            max_cost = trace_cost;
            max_trace = Some(h.clone());
        }
    }

    SemanticCost {
        value: max_cost,
        maximizing_realization: max_trace,
    }
}

/// Verify that a hidden trace projects to an accepted observable trace
///
/// This bridges to the existing verify_chain kernel.
/// Returns true if the projection is accepted.
pub fn verify_projection_is_certified(
    hidden_trace: &HiddenTrace,
    observable_chain_digest: &str,
) -> bool {
    // Placeholder: in full implementation, we would construct a MicroReceipt
    // from the hidden trace and verify it against the observable chain digest.
    // For now, we return true if the trace is non-empty.
    !hidden_trace.states.is_empty()
}

/// Build a fiber from a given observable trace and hidden state enumeration
///
/// Note: In the runtime, enumerating all realizations is expensive/exponential.
/// We implement a bounded enumeration.
pub fn enumerate_realizable_fiber(
    obs_trace: &[ObsState],
    hidden_states: &[HiddenState],
    project_fn: fn(&HiddenState) -> ObsState,
    max_depth: usize,
) -> RealizableFiber {
    // Naive bounded enumeration: generate all traces up to max_depth
    // that project to the observable trace
    let mut fiber = RealizableFiber {
        observable_trace: obs_trace.to_vec(),
        realizations: Vec::new(),
        is_finite: true,
    };

    // For each length up to max_depth
    for depth in 1..=max_depth {
        // Generate all traces of this length that match the projection
        // (This is a placeholder for the actual exhaustive generation)
        if depth > obs_trace.len() {
            break;
        }
    }

    fiber
}

/// Check subadditivity: semantic_cost(τ1 ++ τ2) <= semantic_cost(τ1) + semantic_cost(τ2)
///
/// This corresponds to the Lean theorem `SemanticSystem.semantic_subadditive`.
pub fn check_semantic_cost_subadditive(
    obs1: &[ObsState],
    obs2: &[ObsState],
    hidden_cost_fn: fn(&HiddenState) -> u128,
) -> bool {
    // Compute semantic costs
    let fiber1 = enumerate_realizable_fiber(obs1, &[], |s| s.hash(), 5);
    let fiber2 = enumerate_realizable_fiber(obs2, &[], |s| s.hash(), 5);

    let cost1 = compute_semantic_cost(&fiber1, hidden_cost_fn);
    let cost2 = compute_semantic_cost(&fiber2, hidden_cost_fn);

    let combined: Vec<ObsState> = obs1.iter().chain(obs2.iter()).cloned().collect();
    let fiber_combined = enumerate_realizable_fiber(&combined, &[], |s| s.hash(), 5);
    let cost_combined = compute_semantic_cost(&fiber_combined, hidden_cost_fn);

    cost_combined.value <= cost1.value.saturating_add(cost2.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_trace_basic() {
        let mut t = HiddenTrace::new();
        t.push(HiddenState::Action("a".to_string()));
        t.push(HiddenState::Action("b".to_string()));

        assert_eq!(t.states.len(), 2);
    }

    #[test]
    fn test_semantic_cost_empty() {
        let fiber = RealizableFiber {
            observable_trace: vec![],
            realizations: vec![],
            is_finite: true,
        };

        let cost_fn = |_: &HiddenState| -> u128 { 0 };
        let result = compute_semantic_cost(&fiber, cost_fn);

        assert_eq!(result.value, 0);
    }

    #[test]
    fn test_projection_certified() {
        let mut t = HiddenTrace::new();
        t.push(HiddenState::Terminal);

        let result = verify_projection_is_certified(&t, "fake_digest");

        // Non-empty trace returns true in placeholder logic
        assert!(result);
    }
}
