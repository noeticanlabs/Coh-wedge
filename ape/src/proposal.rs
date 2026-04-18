//! APE Proposal Data Structures
//!
//! Defines the structured candidates for Coh Wedge verification.

use coh_core::types::{MicroReceiptWire, SlabReceiptWire};
use serde::{Deserialize, Serialize};

/// Strategy used to generate the proposal
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    /// Slightly corrupt valid states (original)
    Mutation,
    /// Merge multiple states incorrectly (original)
    Recombination,
    /// Break invariants (original)
    Violation,
    /// Stress numeric boundaries (original)
    Overflow,
    /// Break logical coherence (original)
    Contradiction,
    /// Specification Gaming: satisfies formal rules, violates intent
    /// Example: receipt passes math but encodes wrong outcome
    SpecificationGaming,
    /// Distribution Shift: pushes to rare edge distributions
    /// Example: valid values in weird combinations
    DistributionShift,
    /// Temporal Drift: each step locally valid, global behavior drifts
    /// Example: small consistent bias accumulates over steps
    TemporalDrift,
    /// Ambiguity Exploitation: exploits undefined/optional fields
    /// Example: different interpretations of same receipt
    AmbiguityExploitation,
    /// Adversarial Alignment: appears aligned, passes checks, hides violation
    /// Example: ideal-looking receipt with subtle inconsistency
    AdversarialAlignment,
    /// Non-Termination: creates repeated states or zero-progress cycles
    /// Example: oscillating state transitions that never terminate
    NonTermination,
    /// Livelock: triggers retry storms without resolution
    /// Example: reject → retry → reject cycles
    Livelock,
    /// State Explosion: causes combinatorial growth in verification paths
    /// Example: deeply nested or massively branched structures
    StateExplosion,
    /// Resource Exhaustion: pushes near memory/time/depth limits
    /// Example: maximum chain lengths, near-limit values
    ResourceExhaustion,
    /// Parser Pathology: structurally nasty but superficially plausible inputs
    /// Example: duplicate keys, giant strings, malformed optionals
    ParserPathology,
    /// ShadowChain: individual receipts valid, but chain_digest_prev inconsistent
    /// Example: correct digests individually but wrong lineage connections
    ShadowChain,
    /// GradientDescent: cumulative small pushes toward forbidden regions
    /// Example: each step valid, but accumulation reaches invalid state
    GradientDescent,
    /// OracleManipulation: exploits immutable field assumptions at creation
    /// Example: sets reserved values that appear valid but are semantically wrong
    OracleManipulation,
    /// TypeConfusion: syntactically valid JSON, invalid semantic interpretation
    /// Example: valid numbers that semantically represent impossible values
    TypeConfusion,
    /// ReflexiveAttack: self-referential or circular metadata
    /// Example: receipts that reference themselves or create verification loops
    ReflexiveAttack,
}

impl Strategy {
    pub fn name(&self) -> &'static str {
        match self {
            Strategy::Mutation => "mutation",
            Strategy::Recombination => "recombination",
            Strategy::Violation => "violation",
            Strategy::Overflow => "overflow",
            Strategy::Contradiction => "contradiction",
            Strategy::SpecificationGaming => "spec_gaming",
            Strategy::DistributionShift => "dist_shift",
            Strategy::TemporalDrift => "temporal_drift",
            Strategy::AmbiguityExploitation => "ambiguity",
            Strategy::AdversarialAlignment => "adv_alignment",
            Strategy::NonTermination => "non_term",
            Strategy::Livelock => "livelock",
            Strategy::StateExplosion => "state_bomb",
            Strategy::ResourceExhaustion => "resource_x",
            Strategy::ParserPathology => "parser_bug",
            Strategy::ShadowChain => "shadow_chain",
            Strategy::GradientDescent => "gradient_descent",
            Strategy::OracleManipulation => "oracle_manip",
            Strategy::TypeConfusion => "type_confusion",
            Strategy::ReflexiveAttack => "reflexive",
        }
    }

    /// Human-readable explanation of what this strategy does
    pub fn note(&self) -> &'static str {
        match self {
            Strategy::Mutation => "altered receipt field while preserving surface structure",
            Strategy::Recombination => "spliced valid fragments into invalid chain topology",
            Strategy::Violation => "broke invariant directly",
            Strategy::Overflow => "exceeded bounds or numeric domain assumptions",
            Strategy::Contradiction => "created mutually incompatible claims in one proposal",
            Strategy::SpecificationGaming => "satisfies formal rules but violates intent",
            Strategy::DistributionShift => "pushes to rare edge distributions",
            Strategy::TemporalDrift => "each step valid, global behavior drifts",
            Strategy::AmbiguityExploitation => "exploits undefined or optional fields",
            Strategy::AdversarialAlignment => "appears aligned, hides deeper violation",
            Strategy::NonTermination => "repeated states or zero-progress cycles",
            Strategy::Livelock => "retry storms without resolution",
            Strategy::StateExplosion => "combinatorial growth in verification paths",
            Strategy::ResourceExhaustion => "pushes near memory/time/depth limits",
            Strategy::ParserPathology => "structurally nasty but plausible inputs",
            Strategy::ShadowChain => "valid receipts with wrong chain lineage",
            Strategy::GradientDescent => "tiny pushes accumulating to forbidden regions",
            Strategy::OracleManipulation => "exploits immutable field assumptions",
            Strategy::TypeConfusion => "syntactically valid, semantically wrong",
            Strategy::ReflexiveAttack => "self-referential or circular metadata",
        }
    }

    /// Generate a candidate using this strategy
    /// Note: For now, returns raw Candidate. The metadata is added at the call site.
    pub fn generate(&self, input: &Input, rng: &mut crate::seed::SeededRng) -> Candidate {
        use crate::strategies::ai_failure_modes;
        use crate::strategies::runtime;
        use crate::strategies::{contradiction, mutation, overflow, recombination, violation};
        match self {
            Strategy::Mutation => mutation::run(input, rng),
            Strategy::Recombination => recombination::run(input, rng),
            Strategy::Violation => violation::run(input, rng),
            Strategy::Overflow => overflow::run(input, rng),
            Strategy::Contradiction => contradiction::run(input, rng),
            Strategy::SpecificationGaming => ai_failure_modes::specification_gaming(input, rng),
            Strategy::DistributionShift => ai_failure_modes::distribution_shift(input, rng),
            Strategy::TemporalDrift => ai_failure_modes::temporal_drift(input, rng),
            Strategy::AmbiguityExploitation => ai_failure_modes::ambiguity_exploitation(input, rng),
            Strategy::AdversarialAlignment => ai_failure_modes::adversarial_alignment(input, rng),
            Strategy::NonTermination => runtime::non_termination(input, rng),
            Strategy::Livelock => runtime::livelock(input, rng),
            Strategy::StateExplosion => runtime::state_explosion(input, rng),
            Strategy::ResourceExhaustion => runtime::resource_exhaustion(input, rng),
            Strategy::ParserPathology => runtime::parser_pathology(input, rng),
            Strategy::ShadowChain => crate::strategies::advanced::shadow_chain(input, rng),
            Strategy::GradientDescent => crate::strategies::advanced::gradient_descent(input, rng),
            Strategy::OracleManipulation => {
                crate::strategies::advanced::oracle_manipulation(input, rng)
            }
            Strategy::TypeConfusion => crate::strategies::advanced::type_confusion(input, rng),
            Strategy::ReflexiveAttack => crate::strategies::advanced::reflexive_attack(input, rng),
        }
    }

    /// Get all strategy variants
    pub fn all() -> [Strategy; 20] {
        [
            Strategy::Mutation,
            Strategy::Recombination,
            Strategy::Violation,
            Strategy::Overflow,
            Strategy::Contradiction,
            Strategy::SpecificationGaming,
            Strategy::DistributionShift,
            Strategy::TemporalDrift,
            Strategy::AmbiguityExploitation,
            Strategy::AdversarialAlignment,
            Strategy::NonTermination,
            Strategy::Livelock,
            Strategy::StateExplosion,
            Strategy::ResourceExhaustion,
            Strategy::ParserPathology,
            Strategy::ShadowChain,
            Strategy::GradientDescent,
            Strategy::OracleManipulation,
            Strategy::TypeConfusion,
            Strategy::ReflexiveAttack,
        ]
    }
}

/// Input to the proposal engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    /// Optional base receipt to mutate
    pub base_micro: Option<MicroReceiptWire>,
    /// Optional base chain to mutate
    pub base_chain: Option<Vec<MicroReceiptWire>>,
    /// Optional base slab to mutate  
    pub base_slab: Option<SlabReceiptWire>,
    /// Prompt describes what behavior we want from LLM simulation
    pub prompt: String,
}

impl Input {
    /// Create input from single receipt
    pub fn from_micro(receipt: MicroReceiptWire) -> Self {
        Self {
            base_micro: Some(receipt),
            base_chain: None,
            base_slab: None,
            prompt: String::new(),
        }
    }

    /// Create input from chain
    pub fn from_chain(chain: Vec<MicroReceiptWire>) -> Self {
        Self {
            base_micro: None,
            base_chain: Some(chain),
            base_slab: None,
            prompt: String::new(),
        }
    }

    /// Create input from slab
    pub fn from_slab(slab: SlabReceiptWire) -> Self {
        Self {
            base_micro: None,
            base_chain: None,
            base_slab: Some(slab),
            prompt: String::new(),
        }
    }

    /// Empty input for pure generation
    pub fn empty() -> Self {
        Self {
            base_micro: None,
            base_chain: None,
            base_slab: None,
            prompt: String::new(),
        }
    }

    /// Get base micro receipt if available
    pub fn micro(&self) -> Option<&MicroReceiptWire> {
        self.base_micro.as_ref()
    }

    /// Get base chain if available
    pub fn chain(&self) -> Option<&Vec<MicroReceiptWire>> {
        self.base_chain.as_ref()
    }

    /// Get base slab if available
    pub fn slab(&self) -> Option<&SlabReceiptWire> {
        self.base_slab.as_ref()
    }

    /// Check if input is empty (no base data)
    pub fn is_empty(&self) -> bool {
        self.base_micro.is_none() && self.base_chain.is_none() && self.base_slab.is_none()
    }
}

/// Generated proposal candidate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    /// Description of what was done
    pub prompt: String,
    /// Unique ID (generated hash)
    pub proposal_id: String,
    /// Strategy used
    pub strategy: Strategy,
    /// Seed used for generation
    pub seed: u64,
    /// The candidate data
    pub candidate: Candidate,
}

impl Proposal {
    /// Create new proposal
    pub fn new(strategy: Strategy, seed: u64, candidate: Candidate) -> Self {
        let proposal_id = format!(
            "{:016x}-{:x}",
            seed,
            candidate.content_hash() & 0xFFFFFFFFFFFFF
        );

        Self {
            prompt: format!("{}: seed={}", strategy.name(), seed),
            proposal_id,
            strategy,
            seed,
            candidate,
        }
    }
}

/// Candidate type with data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Candidate {
    /// Single micro receipt
    Micro(MicroReceiptWire),
    /// Chain of receipts
    Chain(Vec<MicroReceiptWire>),
    /// Slab receipt
    Slab(SlabReceiptWire),
}

/// Attack subtypes for mutation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationSubtype {
    /// Whitespace, formatting, non-semantic
    Cosmetic,
    /// Receipt fields, amounts, IDs, timestamps, hashes
    Integrity,
    /// Change one field without updating dependent fields
    Consistency,
    /// Alter issuer/origin/chain identity
    Provenance,
}

impl MutationSubtype {
    pub fn name(&self) -> &'static str {
        match self {
            MutationSubtype::Cosmetic => "cosmetic",
            MutationSubtype::Integrity => "integrity",
            MutationSubtype::Consistency => "consistency",
            MutationSubtype::Provenance => "provenance",
        }
    }
}

/// Attack subtypes for recombination
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecombinationSubtype {
    /// Combines fragments preserving valid lineage
    Benign,
    /// Inserts valid fragment into wrong history
    ChainSplice,
    /// Mixes fragments from different chains/sessions
    CrossOrigin,
    /// Valid parts, invalid order
    SequenceViolation,
    /// Predecessor references don't match
    HashLinkBreak,
}

impl RecombinationSubtype {
    pub fn name(&self) -> &'static str {
        match self {
            RecombinationSubtype::Benign => "benign",
            RecombinationSubtype::ChainSplice => "chain_splice",
            RecombinationSubtype::CrossOrigin => "cross_origin",
            RecombinationSubtype::SequenceViolation => "sequence_violation",
            RecombinationSubtype::HashLinkBreak => "hash_link_break",
        }
    }
}

/// Metadata for candidate (for replayability and explainability)
/// Note: Not serialized - used at runtime for demo output
pub struct CandidateMetadata {
    /// Strategy that generated this candidate
    pub strategy_name: &'static str,
    /// Specific attack type within the strategy
    pub attack_kind: &'static str,
    /// Detailed subtype for mutation/recombination
    pub attack_subtype: Option<&'static str>,
    /// Seed used for generation (for replay)
    pub seed: u64,
    /// Human-readable explanation of the corruption
    pub notes: String,
    /// Whether this should fail (for triage)
    pub should_fail: Option<bool>,
}

impl CandidateMetadata {
    pub fn new(
        strategy_name: &'static str,
        attack_kind: &'static str,
        attack_subtype: Option<&'static str>,
        seed: u64,
        notes: String,
        should_fail: Option<bool>,
    ) -> Self {
        Self {
            strategy_name,
            attack_kind,
            attack_subtype,
            seed,
            notes,
            should_fail,
        }
    }
}

impl Candidate {
    /// Get the internal content hash (for ID generation)
    ///
    /// Not a cryptographic hash - just for uniqueness
    pub fn content_hash(&self) -> u64 {
        use serde_json::to_string;
        match self {
            Candidate::Micro(w) => {
                let s = to_string(w).unwrap_or_default();
                s.len() as u64
            }
            Candidate::Chain(v) => {
                let s = to_string(v).unwrap_or_default();
                s.len() as u64
            }
            Candidate::Slab(w) => {
                let s = to_string(w).unwrap_or_default();
                s.len() as u64
            }
        }
    }

    /// Get micro receipt if present
    pub fn as_micro(&self) -> Option<&MicroReceiptWire> {
        match self {
            Candidate::Micro(w) => Some(w),
            _ => None,
        }
    }

    /// Get chain if present
    pub fn as_chain(&self) -> Option<&Vec<MicroReceiptWire>> {
        match self {
            Candidate::Chain(v) => Some(v),
            _ => None,
        }
    }

    /// Get slab if present
    pub fn as_slab(&self) -> Option<&SlabReceiptWire> {
        match self {
            Candidate::Slab(w) => Some(w),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coh_core::types::MetricsWire;

    fn sample_micro() -> MicroReceiptWire {
        MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            canon_profile_hash: "0".repeat(64),
            policy_hash: "0".repeat(64),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: "0".repeat(64),
            state_hash_next: "0".repeat(64),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "80".to_string(),
                spend: "20".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        }
    }

    #[test]
    fn test_proposal_new() {
        let micro = sample_micro();
        let candidate = Candidate::Micro(micro);
        let proposal = Proposal::new(Strategy::Mutation, 42, candidate);

        assert_eq!(proposal.seed, 42);
        assert_eq!(proposal.strategy, Strategy::Mutation);
    }

    #[test]
    fn test_input_from_micro() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        assert!(input.base_micro.is_some());
        assert!(input.base_chain.is_none());
    }
}
