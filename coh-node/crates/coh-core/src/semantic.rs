//! Semantic Layer - Mirroring Lean Axioms
//!
//! Implements the semantic cost and defect bound checks.
//! Defect Bound: delta(trace) <= defect

use crate::types::{MicroReceipt, RejectCode};

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
    pub fn delta_hat(step_type: &Option<String>) -> Result<(u128, SemanticEnvelopeSource), RejectCode> {
        match step_type.as_deref() {
            Some("coh.step.identity") => Ok((0, SemanticEnvelopeSource::StaticTable)),
            Some("coh.step.transfer") => Ok((5, SemanticEnvelopeSource::RegistryLookup)), 
            Some("coh.step.mint") => Ok((0, SemanticEnvelopeSource::StaticTable)),
            Some("coh.step.burn") => Ok((0, SemanticEnvelopeSource::StaticTable)),
            Some(_) | None => Err(RejectCode::SemanticEnvelopeMissing),
        }
    }

    /// Check if the receipt's defect dominates the conservative envelope: defect >= delta_hat
    pub fn verify_defect_bound(receipt: &MicroReceipt) -> Result<(), RejectCode> {
        let (delta_hat, _source) = Self::delta_hat(&receipt.step_type)?;
        if receipt.metrics.defect >= delta_hat {
            Ok(())
        } else {
            Err(RejectCode::SemanticEnvelopeViolation)
        }
    }

    /// Check if a step is an identity transition.
    /// Lean Axiom: Identity traces have zero cost.
    pub fn is_identity(step_type: &Option<String>) -> bool {
        matches!(step_type, Some(t) if t == "coh.step.identity")
    }
}
