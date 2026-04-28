//! Noetic Proposal Engine (NPE)
//! 
//! Implements Chaos–Coherence Boundary Theory:
//! "Chaos is forward admissible generation."
//! 
//! Law of Chaos: M(g') + C(p) <= M(g) + D(p)

use coh_core::types::{Decision, RejectCode};
use serde::{Deserialize, Serialize};

/// Resource metrics for the Law of Chaos
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChaosMetrics {
    /// M: Generative disorder or unresolved-complexity valuation
    pub disorder: u128,
    /// C: Process cost (generation/search cost)
    pub cost: u128,
    /// D: Generative defect or exploratory slack
    pub slack: u128,
}

impl ChaosMetrics {
    pub fn new(disorder: u128, cost: u128, slack: u128) -> Self {
        Self { disorder, cost, slack }
    }

    /// The Law of Chaos: M(g') + C(p) <= M(g) + D(p)
    /// Returns true if the generation is admissible.
    pub fn is_chaos_admissible(
        prev_disorder: u128,
        next_disorder: u128,
        cost: u128,
        slack: u128,
    ) -> bool {
        let lhs = next_disorder.saturating_add(cost);
        let rhs = prev_disorder.saturating_add(slack);
        lhs <= rhs
    }
}

/// A Chaos Candidate (Forward Generation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChaosCandidate {
    pub prev_state_hash: String,
    pub next_state_hash: String,
    pub metrics: ChaosMetrics,
}

impl ChaosCandidate {
    pub fn is_admissible(&self, prev_disorder: u128) -> bool {
        ChaosMetrics::is_chaos_admissible(
            prev_disorder,
            self.metrics.disorder,
            self.metrics.cost,
            self.metrics.slack,
        )
    }
}

/// The Formation Set: Intersection of Chaos (Generation) and Coherence (Verification)
pub struct FormationResult {
    pub is_chaos_admissible: bool,
    pub is_coherence_admissible: bool,
    pub is_formation_admissible: bool,
    pub chaos_margin: i128,
    pub coherence_margin: i128,
}

impl FormationResult {
    pub fn compute(
        chaos_admissible: bool,
        coherence_admissible: bool,
        chaos_margin: i128,
        coherence_margin: i128,
    ) -> Self {
        Self {
            is_chaos_admissible: chaos_admissible,
            is_coherence_admissible: coherence_admissible,
            is_formation_admissible: chaos_admissible && coherence_admissible,
            chaos_margin,
            coherence_margin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_admissibility_valid() {
        // M(g) = 100, D(p) = 10 -> RHS = 110
        // M(g') = 90, C(p) = 15 -> LHS = 105
        // 105 <= 110 -> OK
        assert!(ChaosMetrics::is_chaos_admissible(100, 90, 15, 10));
    }

    #[test]
    fn test_chaos_admissibility_invalid() {
        // M(g) = 100, D(p) = 5 -> RHS = 105
        // M(g') = 110, C(p) = 10 -> LHS = 120
        // 120 > 105 -> REJECT
        assert!(!ChaosMetrics::is_chaos_admissible(100, 110, 10, 5));
    }

    #[test]
    fn test_chaos_exhaustion_boundary() {
        // M(g) = 100, D(p) = 0 -> RHS = 100
        // M(g') = 100, C(p) = 0 -> LHS = 100
        // 100 <= 100 -> OK (Exact boundary)
        assert!(ChaosMetrics::is_chaos_admissible(100, 100, 0, 0));
    }

    #[test]
    fn test_formation_intersection() {
        let result = FormationResult::compute(true, true, 5, 10);
        assert!(result.is_formation_admissible);

        let result_mixed = FormationResult::compute(true, false, 5, -2);
        assert!(!result_mixed.is_formation_admissible);
        assert!(result_mixed.is_chaos_admissible);
    }
}
