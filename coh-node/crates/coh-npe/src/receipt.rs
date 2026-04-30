//! NPE Boundary Receipts
//!
//! Grounded evidence of verification outcomes and proof closure.

use serde::{Deserialize, Serialize};
use crate::closure::LeanClosureStatus;
use crate::templates::CohTemplateKind;
use crate::failure_taxonomy::FailureReport;

/// Boundary receipt summary consumed by PhaseLoom
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BoundaryReceiptSummary {
    /// Domain (e.g., "code", "test", "docs")
    pub domain: String,
    /// Target (e.g., "function foo", "module bar")
    pub target: String,
    /// Strategy class used (e.g., "synthesize", "refine", "debug")
    pub strategy_class: String,
    /// Coh Template used (e.g., "CertifiedComposition")
    pub coh_template: Option<CohTemplateKind>,
    /// Formal closure status of the proof
    pub closure_status: LeanClosureStatus,
    /// Wildness parameter (0.0 = conservative, 1.0 = aggressive)
    pub wildness: f64,
    /// Genesis margin: M(g') + C(p) - M(g) - D(p)
    pub genesis_margin: i128,
    /// Coherence margin: V_post + spend - V_pre - defect
    pub coherence_margin: i128,
    /// Projection Defect: Coarse-graining slack accounted for in memory
    pub projection_defect: u128,
    /// Algebraic Tension Score (0-100)
    pub tension_score: u128,
    /// Epistemic Provenance: "EXT", "DER", "REP", "SIM"
    pub provenance: String,
    /// [ECOLOGY] Tau of the original record being accessed
    pub record_tau: u64,
    /// [ECOLOGY] Semantic distance to the record
    pub semantic_distance: f64,
    /// Accuracy (Fiber Diameter): lower is more precise
    pub accuracy: f64,
    /// Utility Score for metabolic forgetting
    pub utility: f64,
    /// Whether sorry or admit was detected in the proof
    pub sorry_detected: bool,
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
    /// Detailed failure report from the NPE pipeline
    pub failure_report: Option<FailureReport>,
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
    /// [LORENTZ] The gamma factor (time dilation) observed during the transition
    pub gamma: f64,
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

impl BoundaryReceiptSummary {
    /// Convert to a simulation vector for the dynamic visualization layer.
    pub fn to_simulation_vector(&self) -> serde_json::Value {
        serde_json::json!({
            "target": self.target,
            "strategy": self.strategy_class,
            "accepted": self.accepted,
            "novelty": self.novelty,
            "spend": self.genesis_margin.abs() as f64 / 1000.0, // Scaled for viz
            "defect": self.coherence_margin.abs() as f64 / 1000.0,
            "hash": self.receipt_hash,
            "failure_layer": self.failure_report.as_ref().map(|r| format!("{:?}", r.layer)),
            "failure_kind": self.failure_report.as_ref().map(|r| format!("{:?}", r.kind)),
            "closure": self.closure_status.as_str(),
        })
    }

    /// Is this a valid receipt for strategy update?
    /// Rule: No strategy update without a receipt.
    pub fn is_valid_for_update(&self) -> bool {
        !self.receipt_hash.is_empty() && (self.accepted || self.failure_report.is_some())
    }
}
