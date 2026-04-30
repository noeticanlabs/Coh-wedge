//! NPE Loop Engine
//!
//! Orchestrates the proposal search loop: generate, score, apply to state.

use serde::{Deserialize, Serialize};
use crate::engine::{NpeProposal, NpeError};

/// NPE loop configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeConfig {
    /// Seed for deterministic generation
    pub seed: u64,
    /// Maximum proposals to keep in beam
    pub beam_width: usize,
    /// Maximum mutation depth
    pub max_depth: u32,
    /// Number of parallel candidates to generate
    pub batch_size: usize,
    /// Enable parallel scoring (requires rayon)
    #[cfg(feature = "npe-parallel")]
    pub parallel_scoring: bool,
}

impl Default for NpeConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            beam_width: 10,
            max_depth: 5,
            batch_size: 100,
            #[cfg(feature = "npe-parallel")]
            parallel_scoring: true,
        }
    }
}

/// NPE loop state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeState {
    /// Current configuration
    pub config: NpeConfig,
    /// Current proposals
    pub proposals: Vec<NpeProposal>,
    /// Accepted proposals
    pub accepted: Vec<NpeProposal>,
    /// Rejected proposals with reasons
    pub rejected: Vec<NpeProposal>,
    /// Generation counter
    pub generation: u64,
    /// Last accepted score
    pub best_score: f64,
}

impl NpeState {
    pub fn new(config: NpeConfig) -> Self {
        Self {
            config,
            proposals: Vec::new(),
            accepted: Vec::new(),
            rejected: Vec::new(),
            generation: 0,
            best_score: f64::NEG_INFINITY,
        }
    }

    /// Reset the state
    pub fn reset(&mut self) {
        self.proposals.clear();
        self.accepted.clear();
        self.rejected.clear();
        self.generation = 0;
        self.best_score = f64::NEG_INFINITY;
    }

    /// Add a proposal to the state
    pub fn add_proposal(&mut self, proposal: NpeProposal) {
        if proposal.score > self.best_score {
            self.best_score = proposal.score;
        }
        self.proposals.push(proposal);
    }
}

/// NPE Engine for deterministic proposal generation and search
pub struct NpeEngine {
    pub config: NpeConfig,
}

impl NpeEngine {
    /// Create a new NPE engine with the given configuration
    pub fn new(config: NpeConfig) -> Self {
        Self { config }
    }

    /// Create a new NPE engine with default configuration
    pub fn new_default() -> Self {
        Self {
            config: NpeConfig::default(),
        }
    }

    /// Generate proposals with a given seed.
    /// The generator is deterministic: same seed + same config => same proposals.
    pub fn generate_proposals<G: crate::traits::NpeGenerator>(
        &self,
        count: usize,
        generator: &G,
        ctx: &<G as crate::traits::NpeGenerator>::Context,
    ) -> Result<Vec<NpeProposal>, NpeError> {
        use sha2::{Digest, Sha256};
        use crate::engine::ProposalStatus;

        let mut proposals = Vec::with_capacity(count);
        for i in 0..count {
            let id = format!("p-{:08}-{:04}", self.config.seed, i);
            let content = generator.generate(self.config.seed, i, ctx)?;

            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let content_hash = hex::encode(hasher.finalize());

            proposals.push(NpeProposal {
                id,
                content,
                seed: self.config.seed,
                score: 0.0,
                content_hash,
                depth: 0,
                parent_id: None,
                tau: 0,                        // Default to 0, should be set by ingest or loop
                provenance: "SIM".to_string(), // Default for newly generated
                status: ProposalStatus::Generated,
            });
        }

        Ok(proposals)
    }

    /// Score proposals (advisory only - this ranking is advisory)
    ///
    /// # Important Rule
    ///
    /// Floating-point scores are *strictly advisory*. The final verification must use integer/rational math
    /// via the Coh verifier. This function only ranks proposals for selection.
    pub fn score_proposals<S: crate::traits::NpeScorer>(
        &self,
        proposals: &mut [NpeProposal],
        scorer: &S,
    ) -> Result<(), NpeError> {
        for p in proposals.iter_mut() {
            p.score = scorer.score(p)?;
        }

        // Sort by score (higher is better)
        proposals.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(())
    }

    /// Apply proposals to state
    pub fn apply_to_state(&self, state: &mut NpeState, proposals: Vec<NpeProposal>) {
        state.generation += 1;
        for proposal in proposals {
            state.add_proposal(proposal);
        }
    }

    /// Get the beam (top proposals by score)
    pub fn get_beam<'a>(&self, state: &'a NpeState) -> Vec<&'a NpeProposal> {
        state
            .proposals
            .iter()
            .take(self.config.beam_width)
            .collect()
    }
}
