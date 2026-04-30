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

// Re-export PhaseLoomLite types and functions
pub use fusion_wedge::verify_governed_step;
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
pub use coh_npe::kernel::{NpeKernel, NpeGoverningState, NpeBudget};
pub use coh_core::rv_kernel::{RvKernel, RvGoverningState, ProtectedRvBudget};

// Re-export MathlibAdvisor types and functions
pub use coh_npe::tools::mathlib_advisor::{
    assess_import_risk, check_policy, generate_report, ImportRisk, MathlibAdvisorReport,
    MathlibPolicy, MathlibStrategy,
};

// Re-export key types for convenience
// Note: GenesisCandidate is already defined in this module, so use a prefix
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
pub use causal_cone::*;

/// Level 0: Environmental Envelope (Outer physical limits)
pub struct EnvironmentalEnvelope {
    pub power_mj: Option<u64>,
    pub thermal_headroom_c: Option<f64>,
    pub wallclock_ms: u64,
    pub hardware_available: bool,
    pub network_allowed: bool,
}

/// Level 1: System Reserve (Protected operational limits)
pub struct SystemReserve {
    pub halt_available: bool,
    pub logging_ops: u64,
    pub ledger_append_ops: u64,
    pub recovery_ops: u64,
    pub scheduler_ticks: u64,
}

/// Global Budgets (B_G): Priority-ordered hierarchy
pub struct GlobalBudgets {
    pub env: EnvironmentalEnvelope,
    pub system: SystemReserve,
    pub rv: ProtectedRvBudget,
    pub npe: NpeBudget,
    pub phaseloom: PhaseLoomBudget,
}

/// GMI Step Trace for observability
pub struct GmiStepTrace {
    pub step_id: String,
    pub events: Vec<String>,
    pub decision: Option<RvDecisionKind>,
}

/// Global GMI Governor (Gov_G)
pub struct GmiGovernor {
    pub npe: NpeKernel,
    pub rv: RvKernel,
    pub phaseloom: PhaseLoomKernel,
    pub env: EnvironmentalEnvelope,
    pub system: SystemReserve,
}

impl GmiGovernor {
    pub fn new(
        npe: NpeKernel,
        rv: RvKernel,
        phaseloom: PhaseLoomKernel,
        env: EnvironmentalEnvelope,
        system: SystemReserve,
    ) -> Self {
        Self { npe, rv, phaseloom, env, system }
    }

    /// Whole-System Admissibility Law
    pub fn is_globally_admissible(&self, prev_v: u128, next_v: u128, spend: u128, defect: u128) -> bool {
        let lhs = next_v.saturating_add(spend);
        let rhs = prev_v.saturating_add(defect);
        lhs <= rhs
    }

    /// Execute a governed loop step (Hierarchical Budget Edition)
    pub fn step(&mut self, proposal_id: &str, _content: &str, distance: num_rational::Rational64, c_g: num_rational::Rational64, dt_g: num_rational::Rational64) -> (bool, GmiStepTrace) {
        let mut trace = GmiStepTrace {
            step_id: proposal_id.to_string(),
            events: vec![],
            decision: None,
        };

        // 1. Level 0: Environment Check
        if !self.env.hardware_available || self.env.wallclock_ms == 0 {
            trace.events.push("Governor HALT: Environmental envelope breach".into());
            return (false, trace);
        }

        // 2. Level 1: System Reserve Check
        if !self.system.halt_available || self.system.logging_ops < 10 {
            trace.events.push("Governor HALT: System reserve threatened".into());
            return (false, trace);
        }

        // 3. Level 2: RV Reserve Check
        if !self.rv.can_verify_safely(10) {
            trace.events.push("Governor REJECT: RV reserve protection breach".into());
            return (false, trace);
        }

        // 3.5. Causal Cone Check (Spacelike Rejection)
        let cone_check = classify_gmi_interval(distance, c_g, dt_g);
        if cone_check.class == CausalClass::Spacelike {
            trace.events.push("Governor REJECT: Spacelike causal violation (d_G > c_G * dt_G)".into());
            return (false, trace);
        }

        // [LORENTZ] Calculate the Gamma factor (time dilation)
        let gamma = if cone_check.class == CausalClass::Null {
            100.0 // Cap at boundary
        } else {
            let cg_dt = c_g * dt_g;
            let cg_dt_f = *cg_dt.numer() as f64 / *cg_dt.denom() as f64;
            let ds2_f = *cone_check.interval_sq.numer() as f64 / *cone_check.interval_sq.denom() as f64;
            if ds2_f <= 0.0 {
                100.0
            } else {
                cg_dt_f / ds2_f.sqrt()
            }
        };

        // 4. Pre-state valuation
        let prev_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);

        // 5. PhaseLoom Read (Bias)
        let _bias = match self.phaseloom.get_bias() {
            Ok(b) => {
                trace.events.push("PhaseLoom READ: StrategyBias emitted".into());
                b
            },
            Err(e) => {
                trace.events.push(format!("PhaseLoom REJECT: {}", e));
                return (false, trace);
            }
        };

        // 6. Level 3: NPE Propose
        if !self.npe.is_affordable(100, 1000, 50) {
            trace.events.push("NPE REJECT: Budget exhausted".into());
            return (false, trace);
        }
        trace.events.push("NPE PROPOSE: Candidate knowledge formed".into());

        // 7. Projection Bridge
        let claim = format!("claim_{}", proposal_id);

        // 8. RV Verify
        let decision = self.rv.verify_claim(&claim);
        trace.decision = Some(decision.kind);
        trace.events.push(format!("RV VERIFY: Decision {:?}", decision.kind));

        if decision.kind != RvDecisionKind::Accept {
            return (false, trace);
        }

        // 9. Receipt and Ledger
        self.system.ledger_append_ops = self.system.ledger_append_ops.saturating_sub(1);
        trace.events.push("Ledger APPEND: Receipt emitted".into());

        // 10. PhaseLoom Write
        let receipt = BoundaryReceiptSummary {
            target: proposal_id.to_string(),
            domain: "system".into(),
            accepted: true,
            outcome: "accepted".into(),
            gamma,
            ..Default::default()
        };
        let _ = self.phaseloom.update(&receipt, &PhaseLoomConfig::default());
        trace.events.push("PhaseLoom WRITE: Memory updated from receipt".into());

        // 11. Post-state valuation
        let next_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);

        // 12. Governor post-check: Global Admissibility
        // We allow a defect budget of 100 to account for process cost and tension injection
        if !self.is_globally_admissible(prev_v, next_v, 10, 100) {
            trace.events.push("Governor REJECT: Global admissibility violation".into());
            return (false, trace);
        }

        trace.events.push("Governor COMMIT: Step completed successfully".into());
        (true, trace)
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
