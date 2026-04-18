//! Runtime/Codebase Failure Mode Strategies
//!
//! These strategies target execution-path failures and runtime/codebase issues:
//! - Non-termination: repeated states, cycles, zero-progress
//! - Livelock: retry storms without resolution
//! - State explosion: combinatorial growth in verification paths
//! - Resource exhaustion: memory/time/depth limits
//! - Parser pathology: structurally nasty but plausible inputs

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// Non-Termination: creates candidates with repeated states or zero-progress cycles
/// Tests the "bounded step progress" invariant
pub fn non_termination(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 4 {
        0 => {
            // Oscillation: make state_hash_next == state_hash_prev (stuck state)
            m.state_hash_next = m.state_hash_prev.clone();
            // Don't update chain_digest - this creates zero-progress
        }
        1 => {
            // Duplicate step index (repeated state)
            m.step_index = 0;
        }
        2 => {
            // Zero-value transfer (no progress)
            m.metrics.v_post = m.metrics.v_pre.clone();
            m.metrics.spend = "0".to_string();
        }
        _ => {
            // Self-referential chain digest (cycle)
            m.chain_digest_next = m.chain_digest_prev.clone();
        }
    }

    Candidate::Micro(m)
}

/// Livelock: triggers retry storms without resolution
/// Tests the "retry budget" invariant
pub fn livelock(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 3 {
        0 => {
            // Invalid but recoverable - creates retry loop
            m.metrics.v_pre = "not_a_number".to_string();
        }
        1 => {
            // Near-boundary that keeps failing
            m.metrics.v_post = m.metrics.v_pre.clone();
            m.metrics.spend = "1".to_string();
            // This will fail v_post + spend > v_pre
        }
        _ => {
            // Invalid signature that could be "fixed" and retried
            m.signatures = Some(vec![]);
        }
    }

    Candidate::Micro(m)
}

/// State Explosion: causes combinatorial growth in verification paths
/// Tests the "verification complexity bounded" invariant
pub fn state_explosion(input: &Input, rng: &mut SeededRng) -> Candidate {
    // Create multiple receipts to simulate branching
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    // Generate multiple variants to cause "branch explosion"
    let mut chain = Vec::new();
    for i in 0..5 {
        let mut variant = base.clone();
        variant.step_index = i as u64;
        variant.object_id = format!("{}_branch_{}", variant.object_id, i);
        // Each has slightly different state
        variant.state_hash_next = format!("{:064x}", i as u64);
        chain.push(variant);
    }

    Candidate::Chain(chain)
}

/// Resource Exhaustion: pushes near memory/time/depth limits
/// Tests the "resource budget" invariant
pub fn resource_exhaustion(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 4 {
        0 => {
            // Near-maximum u128
            m.metrics.v_pre = "340282366920938463463374607431768211455".to_string();
            m.metrics.v_post = "0".to_string();
            m.metrics.spend = m.metrics.v_pre.clone();
        }
        1 => {
            // Large step index (could cause iteration issues)
            m.step_index = u64::MAX;
        }
        2 => {
            // Giant string in object_id
            m.object_id = "x".repeat(10000);
        }
        _ => {
            // Large state hashes
            m.state_hash_prev = "f".repeat(64);
            m.state_hash_next = "f".repeat(64);
        }
    }

    Candidate::Micro(m)
}

/// Parser Pathology: structurally nasty but superficially plausible inputs
/// Tests the "parsing must be total/deterministic" invariant
pub fn parser_pathology(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 4 {
        0 => {
            // Whitespace-padded numeric fields (may parse differently)
            m.metrics.v_pre = "  100".to_string();
            m.metrics.v_post = " 80".to_string();
        }
        1 => {
            // Null character in string
            m.object_id = "obj\x00id".to_string();
        }
        2 => {
            // Very long but "valid" string
            m.object_id = "a".repeat(1000);
        }
        _ => {
            // Special characters in version
            m.version = "1.0.0\n".to_string();
        }
    }

    Candidate::Micro(m)
}

/// Generate a base receipt for strategies that need one
fn generate_base(rng: &mut SeededRng) -> MicroReceiptWire {
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;

    let v_pre = 100u128 + (rng.next() as u128 % 100);
    let spend = rng.next() as u128 % 20;
    let v_post = v_pre.saturating_sub(spend);

    let chain_digest_prev_hex = "0".repeat(64);
    let chain_digest_prev = coh_core::types::Hash32::from_hex(&chain_digest_prev_hex)
        .unwrap_or(coh_core::types::Hash32([0u8; 32]));

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("runtime.{}", rng.next() % 100),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: rng.next() as u64 % 10,
        step_type: Some("action".to_string()),
        signatures: Some(vec![]),
        state_hash_prev: format!("{:064x}", rng.next() as u64),
        state_hash_next: format!("{:064x}", rng.next() as u64),
        chain_digest_prev: chain_digest_prev_hex,
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    // Compute valid digest
    if let Ok(r) = coh_core::types::MicroReceipt::try_from(wire.clone()) {
        let prehash = to_prehash_view(&r);
        if let Ok(bytes) = to_canonical_json_bytes(&prehash) {
            let digest = compute_chain_digest(chain_digest_prev, &bytes);
            wire.chain_digest_next = digest.to_hex();
        }
    }

    wire
}
