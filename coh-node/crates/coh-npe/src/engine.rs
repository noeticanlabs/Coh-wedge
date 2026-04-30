//! NPE Proposal and Error Types
//!
//! Core data structures for the Noetican Proposal Engine.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The NPE loop error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NpeError {
    #[error("Seed error: {0}")]
    SeedError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Scoring error: {0}")]
    ScoringError(String),

    #[error("Verification error: {0}")]
    VerificationError(String),

    #[error("Graph error: {0}")]
    GraphError(String),
}

/// The NPE proposal state
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NpeProposal {
    /// Unique identifier for this proposal
    pub id: String,
    /// The semantic content (Lean code, math expression, etc.)
    pub content: String,
    /// The generation seed used
    pub seed: u64,
    /// Current score (advisory, not final)
    pub score: f64,
    /// Proposal hash for dedup
    pub content_hash: String,
    /// Mutation depth from root
    pub depth: u32,
    /// Parent proposal ID (if any)
    pub parent_id: Option<String>,
    /// [ECOLOGY] Creation time in PhaseLoom tau
    pub tau: u64,
    /// [ECOLOGY] Epistemic provenance
    pub provenance: String,
    /// Status
    pub status: ProposalStatus,
}

/// Proposal status in the NPE loop
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProposalStatus {
    /// Generated, pending scoring
    #[default]
    Generated,
    /// Scored, pending verification
    Scored,
    /// Sent to verifier
    Verifying,
    /// Accepted by verifier
    Accepted,
    /// Rejected by verifier
    Rejected(String),
    /// Failed in generation
    Failed(String),
}
