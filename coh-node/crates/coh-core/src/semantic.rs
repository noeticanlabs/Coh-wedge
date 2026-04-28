//! Semantic Layer - Mirroring Lean Axioms
//!
//! Implements the semantic cost and defect bound checks.
//! Defect Bound: delta(trace) <= defect

use crate::types::MicroReceipt;

/// Source of a semantic envelope bound
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticEnvelopeSource {
    StaticTable,
    RegistryLookup,
    TrajectoryEngine,
    ExternalCertificate,
}

/// Registry for semantic delta_hat values (conservative cost envelopes)
pub struct SemanticRegistry;

impl SemanticRegistry {
    /// Get the conservative minimum cost envelope (delta_hat) for a given step type.
    /// The kernel obligation is: delta_hat >= delta (the true hidden cost).
    pub fn delta_hat(step_type: &Option<String>) -> (u128, SemanticEnvelopeSource) {
        match step_type {
            Some(t) => match t.as_str() {
                "coh.step.identity" => (0, SemanticEnvelopeSource::StaticTable),
                "coh.step.transfer" => (5, SemanticEnvelopeSource::RegistryLookup), 
                "coh.step.mint" => (0, SemanticEnvelopeSource::StaticTable),
                "coh.step.burn" => (0, SemanticEnvelopeSource::StaticTable),
                _ => (0, SemanticEnvelopeSource::RegistryLookup),
            },
            None => (0, SemanticEnvelopeSource::StaticTable),
        }
    }

    /// Check if the receipt's defect dominates the conservative envelope: defect >= delta_hat
    pub fn verify_defect_bound(receipt: &MicroReceipt) -> bool {
        let (delta_hat, _source) = Self::delta_hat(&receipt.step_type);
        receipt.metrics.defect >= delta_hat
    }

    /// Check if a step is an identity transition.
    /// Lean Axiom: Identity traces have zero cost.
    pub fn is_identity(step_type: &Option<String>) -> bool {
        matches!(step_type, Some(t) if t == "coh.step.identity")
    }
}
