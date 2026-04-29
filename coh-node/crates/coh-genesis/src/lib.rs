//! Genesis Engine (Forward Generation)
//!
//! Implements the Physics of Assertion:
//! "Genesis is forward admissible generation."
//!
//! Law of Genesis: M(g') + C(p) <= M(g) + D(p)
//!
//! ## Modules
//!
//! - [`candidate`]: Genesis candidate structures and core functions
//! - [`generator`]: Synthetic NPE generator for wildness testing
//! - [`sweep`]: Wildness sweep algorithm
//! - [`report`]: Report generation and exports

use serde::{Deserialize, Serialize};

/// Resource metrics for the Law of Genesis
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisMetrics {
    /// M: Generative complexity or resolution-cost valuation
    pub disorder: u128,
    /// C: Process cost (generation/search cost)
    pub cost: u128,
    /// D: Generative slack or exploratory budget
    pub slack: u128,
}

impl GenesisMetrics {
    pub fn new(disorder: u128, cost: u128, slack: u128) -> Self {
        Self {
            disorder,
            cost,
            slack,
        }
    }

    /// The Law of Genesis: M(g') + C(p) <= M(g) + D(p)
    /// Returns true if the generation is admissible.
    /// Uses checked arithmetic to prevent boundary breaches.
    pub fn is_genesis_admissible(
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

/// A Genesis Candidate (Forward Generation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisCandidate {
    pub prev_state_hash: String,
    pub next_state_hash: String,
    pub metrics: GenesisMetrics,
}

impl GenesisCandidate {
    pub fn is_admissible(&self, prev_disorder: u128) -> bool {
        GenesisMetrics::is_genesis_admissible(
            prev_disorder,
            self.metrics.disorder,
            self.metrics.cost,
            self.metrics.slack,
        )
    }
}

/// The Formation Set: Intersection of Genesis (Generation) and Coherence (Verification)
pub struct FormationResult {
    pub is_genesis_admissible: bool,
    pub is_coherence_admissible: bool,
    pub is_formation_admissible: bool,
}

impl FormationResult {
    pub fn compute(genesis_admissible: bool, coherence_admissible: bool) -> Self {
        Self {
            is_genesis_admissible: genesis_admissible,
            is_coherence_admissible: coherence_admissible,
            is_formation_admissible: genesis_admissible && coherence_admissible,
        }
    }
}

// Re-export modules for NPE Wildness Boundary Test
pub mod candidate;
pub mod code_patch;
pub mod generator;
pub mod report;
pub mod sweep;

// Re-export key types for convenience
// Note: GenesisCandidate is already defined in this module, so use a prefix
pub use candidate::{
    GenesisCandidate as NpeGenesisCandidate, ProjectedCohClaim, WildnessLevel, WildnessResult,
};
pub use code_patch::{
    build_formation_result, check_hard_gates, compute_coherence_metrics, compute_genesis_metrics,
    compute_patch_scores, is_formation_admissible, patch_type_for_wildness, CodePatchCandidate,
    CodePatchFirstFailure, CodePatchFormationResult, CodePatchReport, PatchHardGate, PatchPolicy,
    PatchSelectorMode, RejectPathImpact, RejectPolicyMode,
};
pub use generator::SyntheticNpeGenerator;
pub use report::{
    export_csv, export_json, print_boundary_margin_stats, print_boundary_seeker_result,
    print_first_failure_table, print_rejection_breakdown, print_reproducibility_info,
    print_results_table, print_summary,
};
pub use sweep::{
    find_boundary_seeker, find_edge_seeker, find_near_boundary_candidate, find_optimal_wildness,
    run_wildness_sweep, standard_levels, SweepConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Basic Genesis admissibility - generation should be admissible when disorder decreases
    #[test]
    fn test_genesis_admissible_decreasing_disorder() {
        let prev_disorder = 1000;
        let next_disorder = 800; // Decreased disorder (more coherent)
        let cost = 50;
        let slack = 100;

        let is_admissible =
            GenesisMetrics::is_genesis_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            is_admissible,
            "Generation with decreasing disorder should be admissible"
        );
    }

    /// Test: Genesis admissibility with increased disorder but within slack budget
    #[test]
    fn test_genesis_admissible_with_slack() {
        let prev_disorder = 1000;
        let next_disorder = 1100;
        let cost = 150;
        let slack = 300; // 1100 + 150 = 1250 <= 1000 + 300 = 1300

        let is_admissible =
            GenesisMetrics::is_genesis_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            is_admissible,
            "Generation with sufficient slack should be admissible"
        );
    }

    /// Test: Genesis boundary case - exactly at boundary
    #[test]
    fn test_genesis_boundary_case() {
        let _prev_disorder = 1000;
        let _next_disorder = 900;
        let _cost = 100;
        let _slack = 0;

        // 900 + 100 = 1000
        assert!(GenesisMetrics::is_genesis_admissible(1000, 900, 100, 0));
    }

    /// Test: Non-admissible generation - disorder increases too much
    #[test]
    fn test_genesis_not_admissible() {
        let prev_disorder = 1000;
        let next_disorder = 1300; // Huge increase
        let cost = 100;
        let slack = 50; // Not enough slack

        let is_admissible =
            GenesisMetrics::is_genesis_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(
            !is_admissible,
            "Generation with insufficient slack should not be admissible"
        );
    }

    /// Test: Formation result - both genesis and coherence admissible
    #[test]
    fn test_formation_both_admissible() {
        let result = FormationResult::compute(true, true);

        assert!(result.is_genesis_admissible);
        assert!(result.is_coherence_admissible);
        assert!(result.is_formation_admissible);
    }

    /// Test: GenesisCandidate admissibility check
    #[test]
    fn test_genesis_candidate_admissible() {
        let candidate = GenesisCandidate {
            prev_state_hash: "abc123".to_string(),
            next_state_hash: "def456".to_string(),
            metrics: GenesisMetrics::new(800, 50, 200),
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
            GenesisMetrics::is_genesis_admissible(prev_disorder, next_disorder, cost, slack);

        assert!(!is_admissible);
    }
}
