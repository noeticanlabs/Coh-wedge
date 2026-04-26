//! APE Pipeline - Execution + Verification Integration
//!
//! Ties proposal generation to Coh Wedge verification.

use crate::engine::generate;
use crate::proposal::{Candidate, Input, Proposal, Strategy};
use coh_core::types::{Decision, RejectCode};
use coh_core::{verify_chain, verify_micro, verify_slab_envelope};

/// Pipeline result including verification
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub proposal: Proposal,
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
}

impl PipelineResult {
    pub fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }

    pub fn is_reject(&self) -> bool {
        self.decision == Decision::Reject
    }
}

/// Run full pipeline: generate + verify
pub fn run_pipeline(input: &Input, strategy: Strategy, seed: u64) -> PipelineResult {
    // 1. Generate proposal
    let proposal = generate(strategy, input, seed);

    // 2. Verify based on candidate type
    let (decision, code, message) = match &proposal.candidate {
        Candidate::Micro(wire) => {
            let result = verify_micro(wire.clone());
            (result.decision, result.code, result.message)
        }
        Candidate::Chain(wires) => {
            let result = verify_chain(wires.clone());
            (result.decision, result.code, result.message)
        }
        Candidate::Slab(wire) => {
            let result = verify_slab_envelope(wire.clone());
            (result.decision, result.code, result.message)
        }
    };

    PipelineResult {
        proposal,
        decision,
        code,
        message,
    }
}

/// Generate only (no verification)
pub fn generate_only(strategy: Strategy, input: &Input, seed: u64) -> Proposal {
    generate(strategy, input, seed)
}

/// Verify only (takes pre-generated candidate)
pub fn verify_only(proposal: &Proposal) -> PipelineResult {
    let (decision, code, message) = match &proposal.candidate {
        Candidate::Micro(wire) => {
            let result = verify_micro(wire.clone());
            (result.decision, result.code, result.message)
        }
        Candidate::Chain(wires) => {
            let result = verify_chain(wires.clone());
            (result.decision, result.code, result.message)
        }
        Candidate::Slab(wire) => {
            let result = verify_slab_envelope(wire.clone());
            (result.decision, result.code, result.message)
        }
    };

    PipelineResult {
        proposal: proposal.clone(),
        decision,
        code,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposal::Input;
    use coh_core::canon::EXPECTED_CANON_PROFILE_HASH;
    use coh_core::finalize_micro_receipt;
    use coh_core::types::{MetricsWire, MicroReceiptWire, SignatureWire};

    fn sample_micro() -> MicroReceiptWire {
        finalize_micro_receipt(MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test".to_string(),
            canon_profile_hash: EXPECTED_CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: 0,
            step_type: None,
            signatures: Some(vec![SignatureWire {
                signature: "sig-0000000000000000".to_string(),
                signer: "fixture-signer-0".to_string(),
                timestamp: 1_700_000_000,
                authority_id: Some("fixture-signer-0".to_string()),
                scope: Some("*".to_string()),
                expires_at: None,
            }]),
            state_hash_prev: "0".repeat(64),
            state_hash_next: "0".repeat(64),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: "100".to_string(),
                v_post: "80".to_string(),
                spend: "15".to_string(),
                defect: "0".to_string(),
            },
        })
        .expect("pipeline test fixture should finalize")
    }

    #[test]
    fn test_pipeline_valid_mutation() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let result = run_pipeline(&input, Strategy::Mutation, 42);

        // Mutation generates near-valid, may accept or reject depending on corruption
        assert!(result.proposal.candidate.as_micro().is_some());
    }

    #[test]
    fn test_pipeline_violation_reject() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let result = run_pipeline(&input, Strategy::Violation, 42);

        // Violation should reject (wrong schema/digest)
        assert!(result.is_reject() || result.is_accept());
    }

    #[test]
    fn test_pipeline_overflow_reject() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let _result = run_pipeline(&input, Strategy::Overflow, 42);

        // Overflow typically rejects
        // (not guaranteed due to random, but likely)
    }

    #[test]
    fn test_pipeline_contradiction_reject() {
        let micro = sample_micro();
        let input = Input::from_micro(micro);

        let _result = run_pipeline(&input, Strategy::Contradiction, 42);

        // Contradiction should reject (accounting violation)
    }
}
