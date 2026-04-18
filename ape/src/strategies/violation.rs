//! Violation Strategy
//!
//! Breaks invariants to test verification rejection.

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// Run violation strategy
pub fn run(input: &Input, rng: &mut SeededRng) -> Candidate {
    if let Some(ref micro) = input.base_micro {
        violate_micro(micro, rng)
    } else if let Some(ref chain) = input.base_chain {
        violate_chain(chain, rng)
    } else {
        generate_violation(rng)
    }
}

fn violate_micro(wire: &MicroReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut m = wire.clone();

    // Pick violation type
    match rng.next() % 4 {
        0 => {
            // Wrong schema ID
            m.schema_id = "invalid.schema".to_string();
        }
        1 => {
            // Wrong version
            m.version = "9.9.9".to_string();
        }
        2 => {
            // Broken digest (will fail RejectChainDigest)
            m.chain_digest_next = "deadbeef".repeat(8);
        }
        _ => {
            // Broken state link (will fail RejectStateHashLink)
            m.state_hash_prev = "badbad".repeat(16);
        }
    }

    Candidate::Micro(m)
}

fn violate_chain(chain: &[MicroReceiptWire], rng: &mut SeededRng) -> Candidate {
    if chain.is_empty() {
        return generate_violation(rng);
    }

    let mut new_chain = chain.to_vec();
    let idx = rng.next_index(new_chain.len());

    // Violate at random position
    match rng.next() % 3 {
        0 => {
            new_chain[idx].schema_id = "wrong.schema".to_string();
        }
        1 => {
            new_chain[idx].chain_digest_next = "0".repeat(64); // Invalid digest
        }
        _ => {
            new_chain[idx].state_hash_prev = "deadbeef".repeat(8);
        }
    }

    Candidate::Chain(new_chain)
}

fn generate_violation(rng: &mut SeededRng) -> Candidate {
    let step = rng.next() as u64;

    // Create receipt with wrong schema (violation)
    let wire = MicroReceiptWire {
        schema_id: "invalid.schema.v1".to_string(),
        version: "99.99.99".to_string(),
        object_id: format!("ape.violation.{}", step),
        canon_profile_hash: "baadf00d".repeat(8),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("violation".to_string()),
        signatures: None,
        state_hash_prev: "0".repeat(64),
        state_hash_next: "0".repeat(64),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64), // Invalid digest
        metrics: coh_core::types::MetricsWire {
            v_pre: "100".to_string(),
            v_post: "80".to_string(),
            spend: "15".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    Candidate::Micro(wire)
}
