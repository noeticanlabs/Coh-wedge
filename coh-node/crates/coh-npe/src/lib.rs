//! Noetic Proposal Engine (NPE)
//!
//! Implements Chaos–Coherence Boundary Theory:
//! "Chaos is forward admissible generation."
//!
//! Law of Chaos: M(g') + C(p) <= M(g) + D(p)

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
        Self {
            disorder,
            cost,
            slack,
        }
    }

    /// The Law of Chaos: M(g') + C(p) <= M(g) + D(p)
    /// Returns true if the generation is admissible.
    /// Uses checked arithmetic to prevent boundary breaches.
    pub fn is_chaos_admissible(
        prev_disorder: u128,
        next_disorder: u128,
        cost: u128,
        slack: u128,
    ) -> bool {
        let lhs = next_disorder.checked_add(cost);
        let rhs = prev_disorder.checked_add(slack);
        match (lhs, rhs) {
            (Some(l), Some(r)) => l <= r,
            _ => false, // Overflow rejects
        }
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
}

impl FormationResult {
    pub fn compute(
        chaos_admissible: bool,
        coherence_admissible: bool,
    ) -> Self {
        Self {
            is_chaos_admissible: chaos_admissible,
            is_coherence_admissible: coherence_admissible,
            is_formation_admissible: chaos_admissible && coherence_admissible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Basic Chaos admissibility - generation should be admissible when disorder decreases
    #[test]
    fn test_chaos_admissible_decreasing_disorder() {
        let prev_disorder = 1000;
        let next_disorder = 800; // Decreased disorder (more coherent)
        let cost = 50;
        let slack = 100;

        let is_admissible =
            ChaosMetrics::is_chaos_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            is_admissible,
            "Generation with decreasing disorder should be admissible"
        );
    }

    /// Test: Chaos admissibility with increased disorder but within slack budget
    #[test]
    fn test_chaos_admissible_with_slack() {
        let prev_disorder = 1000;
        let next_disorder = 1100; 
        let cost = 150;
        let slack = 300; // 1100 + 150 = 1250 <= 1000 + 300 = 1300

        let is_admissible =
            ChaosMetrics::is_chaos_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            is_admissible,
            "Generation with sufficient slack should be admissible"
        );
    }

    /// Test: Chaos boundary case - exactly at boundary
    #[test]
    fn test_chaos_boundary_case() {
        let _prev_disorder = 1000;
        let _next_disorder = 900;
        let _cost = 50;
        let _slack = -50i128; // Wait, slack is u128. Let's use 0.
        
        // 900 + 100 = 1000
        assert!(ChaosMetrics::is_chaos_admissible(1000, 900, 100, 0));
    }

    /// Test: Non-admissible generation - disorder increases too much
    #[test]
    fn test_chaos_not_admissible() {
        let prev_disorder = 1000;
        let next_disorder = 1300; // Huge increase
        let cost = 100;
        let slack = 50; // Not enough slack

        let is_admissible =
            ChaosMetrics::is_chaos_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            !is_admissible,
            "Generation with insufficient slack should not be admissible"
        );
    }

    /// Test: Formation result - both chaos and coherence admissible
    #[test]
    fn test_formation_both_admissible() {
        let result = FormationResult::compute(true, true);

        assert!(result.is_chaos_admissible);
        assert!(result.is_coherence_admissible);
        assert!(result.is_formation_admissible);
    }

    /// Test: ChaosCandidate admissibility check
    #[test]
    fn test_chaos_candidate_admissible() {
        let candidate = ChaosCandidate {
            prev_state_hash: "abc123".to_string(),
            next_state_hash: "def456".to_string(),
            metrics: ChaosMetrics::new(800, 50, 200),
        };

        let prev_disorder = 1000;
        let is_admissible = candidate.is_admissible(prev_disorder);

        assert!(is_admissible);
    }

    /// Test: Overflow protection with checked arithmetic
    #[test]
    fn test_overflow_protection() {
        let prev_disorder = u128::MAX;
        let next_disorder = u128::MAX;
        let cost = 1;
        let slack = 0;

        // u128::MAX + 1 overflows. Should reject.
        let is_admissible =
            ChaosMetrics::is_chaos_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(!is_admissible);
    }
}
