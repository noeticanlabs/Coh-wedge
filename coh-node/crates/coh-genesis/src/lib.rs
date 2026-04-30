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
pub mod fusion_wedge;
pub mod math_analytic_failure;
pub mod npe_verifier_integration;
pub mod proof_receipt;
pub mod report;
pub mod sweep;
pub mod governor_tests;
pub mod lean_json_export;
pub mod causal_cone;
pub mod ledger;
pub mod verifier_tools;
pub mod atom;
pub use atom::GmiAtom;
pub mod kernel_invariants;

// Re-export PhaseLoomLite types and functions
pub use fusion_wedge::verify_governed_step;
pub use coh_phaseloom as phaseloom_lite;
pub use coh_phaseloom::{
    phaseloom_circuit_broken, phaseloom_ingest, phaseloom_init, phaseloom_sample,
    phaseloom_serialize, PhaseLoomConfig, PhaseLoomState,
};
pub use coh_phaseloom::kernel::PhaseLoomKernel;
pub use coh_phaseloom::budget::PhaseLoomBudget;
pub use coh_npe::{
    BoundaryReceiptSummary, MathlibEffect, StrategyWeights, LeanClosureStatus,
    NpeError, NpeProposal, ProposalStatus, NpeConfig, NpeEngine, NpeState,
};
pub mod npe {
    pub use coh_npe::*;
}
pub use coh_npe::kernel::{NpeKernel, NpeGoverningState, NpeBudget};
pub use coh_core::rv_kernel::{RvKernel, RvGoverningState, ProtectedRvBudget};
pub mod rv {
    pub use coh_core::rv_kernel::*;
}

pub use coh_npe::tools::code_patch;
pub use coh_npe::tools::mathlib_advisor;
pub use coh_npe::tools::lean_proof;

// Re-export key types for convenience
pub use coh_npe::candidate::{
    GenesisCandidate as NpeGenesisCandidate, ProjectedCohClaim, WildnessLevel, WildnessResult,
};

pub use coh_npe::tools::code_patch::{
    build_formation_result, check_hard_gates, compute_coherence_metrics, compute_genesis_metrics,
    compute_patch_scores, is_formation_admissible, patch_type_for_wildness, CodePatchCandidate,
    CodePatchFirstFailure, CodePatchFormationResult, CodePatchReport, PatchHardGate, PatchPolicy,
    PatchSelectorMode, RejectPathImpact, RejectPolicyMode,
    // NEW: Dependency upgrades
    check_cargo_outdated, generate_dep_upgrade_text, is_upgrade_admissible, 
    parse_cargo_toml, UpgradeClass, UpgradeTarget, DependencyUpgradeCandidate, 
    DependencyUpgradeReport, CrateUpdate, ParsedDependency,
};
pub use coh_npe::generator::SyntheticNpeGenerator;
pub use report::{
    export_csv, export_json, print_boundary_margin_stats, print_boundary_seeker_result,
    print_first_failure_table, print_rejection_breakdown, print_reproducibility_info,
    print_results_table, print_summary,
};
pub use sweep::{
    find_boundary_seeker, find_edge_seeker, find_near_boundary_candidate, find_optimal_wildness,
    run_wildness_sweep, standard_levels, SweepConfig,
};

use coh_core::rv_kernel::RvDecisionKind;
use coh_core::types::FormalStatus;
pub use causal_cone::*;

/// Level 0: Environmental Envelope (Outer physical limits)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalEnvelope {
    pub power_mj: Option<u64>,
    pub thermal_headroom_c: Option<f64>,
    pub wallclock_ms: u64,
    pub hardware_available: bool,
    pub network_allowed: bool,
}

/// Level 1: System Reserve (Protected operational limits)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemReserve {
    pub halt_available: bool,
    pub logging_ops: u64,
    pub ledger_append_ops: u64,
    pub recovery_ops: u64,
    pub scheduler_ticks: u64,
}

/// Global Budgets (B_G): Priority-ordered hierarchy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalBudgets {
    pub env: EnvironmentalEnvelope,
    pub system: SystemReserve,
    pub rv: ProtectedRvBudget,
    pub npe: NpeBudget,
    pub phaseloom: PhaseLoomBudget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GmiStepOutcome {
    CommittedWithMemoryUpdate,
    CommittedMemorySkipped(String),
    Rejected(String),
    Deferred(String),
    SafeHalt(String),
}

use coh_core::cohbit::CohBitState;

/// GMI Step Trace for observability
#[derive(Debug, Serialize, Deserialize)]
pub struct GmiStepTrace {
    pub step_id: String,
    pub events: Vec<String>,
    pub decision: Option<RvDecisionKind>,
    pub outcome: Option<GmiStepOutcome>,
    pub cohbit_state: CohBitState,
}

/// Global GMI Governor (Gov_G)
/// Now implemented as a wrapper around the GmiAtom.
pub struct GmiGovernor {
    pub atom: atom::GmiAtom,
}

use coh_physics::CohSpinor;

impl GmiGovernor {
    pub fn new(
        npe: NpeKernel,
        rv: RvKernel,
        phaseloom: PhaseLoomKernel,
        env: EnvironmentalEnvelope,
        system: SystemReserve,
        carrier: Option<CohSpinor>,
    ) -> Self {
        let budgets = GlobalBudgets {
            env,
            system,
            rv: rv.budget.clone(),
            npe: npe.budget.clone(),
            phaseloom: phaseloom.budget.clone(),
        };
        Self { 
            atom: atom::GmiAtom::new(npe, rv, phaseloom, budgets, carrier)
        }
    }

    /// Whole-System Admissibility Law
    pub fn is_globally_admissible(&self, prev_v: u128, next_v: u128, spend: u128, defect: u128) -> bool {
        self.atom.is_stable(prev_v, next_v, spend, defect)
    }

    /// Execute a governed loop step (Hierarchical Budget Edition)
    pub fn step(
        &mut self, 
        proposal_id: &str, 
        content: &str, 
        distance: num_rational::Rational64, 
        c_g: num_rational::Rational64, 
        dt_g: num_rational::Rational64, 
        formal_status: FormalStatus
    ) -> (bool, GmiStepTrace) {
        self.atom.emit_cohbit(proposal_id, content, distance, c_g, dt_g, formal_status)
    }
}

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
