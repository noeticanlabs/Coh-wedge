//! AI Failure Mode Strategies
//!
//! These strategies target known AI failure modes:
//! - Specification Gaming: satisfies formal rules, violates intent
//! - Distribution Shift: pushes to rare edge distributions
//! - Temporal Drift: each step valid, global behavior drifts
//! - Ambiguity Exploitation: exploits undefined/optional fields
//! - Adversarial Alignment: appears aligned, hides deeper violation

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// Specification Gaming: satisfies formal rules but violates intent
/// Example: receipt passes math (v_post + spend <= v_pre) but encodes "did nothing useful"
/// Intent violation: v_post = 0 means "no value transferred" but mathematically valid
pub fn specification_gaming(input: &Input, rng: &mut SeededRng) -> Candidate {
    // Get base receipt to mutate
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    // Specification gaming: make v_post = 0 but spend = 0 (did nothing, but valid)
    // Or: make small spend but no actual value change
    let mut m = base.clone();

    match rng.next() % 3 {
        0 => {
            // v_post = 0 (looks like "nothing done" but passes math)
            m.metrics.v_post = "0".to_string();
            m.metrics.spend = "0".to_string(); // No spend
        }
        1 => {
            // Near-zero values that pass bounds but are meaningless
            m.metrics.v_post = "1".to_string();
            m.metrics.spend = "0".to_string();
            m.metrics.v_pre = "1".to_string(); // tiny value
        }
        _ => {
            // Swap v_pre and v_post (same total, but wrong direction)
            let v_pre = m.metrics.v_pre.clone();
            m.metrics.v_pre = m.metrics.v_post.clone();
            m.metrics.v_post = v_pre;
            m.metrics.spend = "0".to_string(); // neutralize spend
        }
    }

    Candidate::Micro(m)
}

/// Distribution Shift: pushes to rare edge distributions
/// Example: extreme values, unusual combinations that still pass checks
pub fn distribution_shift(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 4 {
        0 => {
            // Maximum values (edge of numeric bounds)
            m.metrics.v_pre = "340282366920938463463374607431768211455".to_string(); // near u128 max
            m.metrics.v_post = "0".to_string();
            m.metrics.spend = m.metrics.v_pre.clone(); // spend everything
        }
        1 => {
            // Very small values (could cause precision issues)
            m.metrics.v_pre = "1".to_string();
            m.metrics.v_post = "0".to_string();
            m.metrics.spend = "1".to_string();
        }
        2 => {
            // Unusual ratio: v_post >> v_pre (should fail, but let's see)
            // Actually this would fail - make it pass by making them equal
            m.metrics.v_pre = "100".to_string();
            m.metrics.v_post = "100".to_string();
            m.metrics.spend = "0".to_string();
        }
        _ => {
            // Round number patterns that might confuse parsers
            m.metrics.v_pre = "999999999999999999".to_string();
            m.metrics.v_post = "999999999999999998".to_string();
            m.metrics.spend = "1".to_string();
        }
    }

    Candidate::Micro(m)
}

/// Temporal Drift: each step locally valid, but global behavior drifts
/// For single receipt: simulates "step in a chain where accumulation would break"
/// This creates a receipt that looks like middle of drifted chain
pub fn temporal_drift(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    // Simulate accumulated drift: slightly biased values that seem ok individually
    // but represent gradual value leakage
    match rng.next() % 3 {
        0 => {
            // Small positive defect that accumulates
            m.metrics.defect = "1".to_string();
            // Make it look like v_post should be higher given spend
            let v_pre: u128 = m.metrics.v_pre.parse().unwrap_or(100);
            let spend: u128 = m.metrics.spend.parse().unwrap_or(0);
            let expected_v_post = v_pre.saturating_sub(spend);
            // Set v_post slightly lower than expected (small leakage)
            m.metrics.v_post = (expected_v_post.saturating_sub(1)).to_string();
        }
        1 => {
            // Slightly wrong chain position that accumulates
            let step = m.step_index;
            m.step_index = step.wrapping_add(5); // Skip steps - local valid, breaks chain
        }
        _ => {
            // Metrics drift: spend more than "appears" from values
            m.metrics.spend = "10".to_string();
            m.metrics.v_pre = "20".to_string();
            m.metrics.v_post = "15".to_string(); // v_post + spend = 25 > v_pre = 20!
                                                 // This actually might fail - let's make it pass
            m.metrics.v_post = "10".to_string(); // v_post + spend = 20 = v_pre (valid)
                                                 // But semantically wrong - spend = 10 but only moved 5 value
            m.metrics.defect = "5".to_string(); // hidden defect
        }
    }

    Candidate::Micro(m)
}

/// Ambiguity Exploitation: exploits undefined/optional fields
/// Example: missing signatures, ambiguous step_type, etc.
pub fn ambiguity_exploitation(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 4 {
        0 => {
            // Ambiguous step_type (None vs Some)
            m.step_type = None;
        }
        1 => {
            // Empty but valid-looking strings
            m.object_id = "".to_string();
        }
        2 => {
            // Unusual but valid schema version
            m.version = "0.0.0".to_string();
        }
        _ => {
            // Extra whitespace in numeric fields (might parse differently)
            m.metrics.v_pre = " 100".to_string();
            m.metrics.v_post = " 80".to_string();
            // This would likely fail parsing - let's make it valid
            m.metrics.v_pre = "100".to_string();
            m.metrics.v_post = "80".to_string();
            // But wrong formatting in canonicalization
            m.state_hash_prev = "  ".to_string().repeat(32);
        }
    }

    Candidate::Micro(m)
}

/// Adversarial Alignment: appears highly aligned, passes checks, hides deeper violation
/// Creates "best looking" receipt with subtle hidden inconsistency
pub fn adversarial_alignment(input: &Input, rng: &mut SeededRng) -> Candidate {
    let base = input
        .base_micro
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_base(rng));

    let mut m = base.clone();

    match rng.next() % 3 {
        0 => {
            // Looks perfect: all values correct, but hidden hash mismatch
            m.metrics.v_pre = "100".to_string();
            m.metrics.v_post = "80".to_string();
            m.metrics.spend = "20".to_string();
            // But chain_digest doesn't match computed
            m.chain_digest_next = "deadbeef".repeat(8);
        }
        1 => {
            // Perfect state hashes but wrong semantic
            m.state_hash_prev = format!("{:064x}", 1u64);
            m.state_hash_next = format!("{:064x}", 2u64); // Looks sequential
                                                          // But chain_digest doesn't match
            m.chain_digest_next = "cafebabe".repeat(8);
        }
        _ => {
            // All signatures present (looks good) but wrong content
            m.signatures = Some(vec![]); // Empty signatures array
            m.object_id = "aligned_perfect_agent".to_string();
            // But other fields corrupted
            m.step_index = 999999;
        }
    }

    Candidate::Micro(m)
}

/// Generate a base receipt for strategies that need one
fn generate_base(rng: &mut SeededRng) -> MicroReceiptWire {
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;
    use std::convert::TryFrom;

    let v_pre = 100u128 + (rng.next() as u128 % 100);
    let spend = rng.next() as u128 % 20;
    let v_post = v_pre.saturating_sub(spend);

    let chain_digest_prev_hex = "0".repeat(64);
    let chain_digest_prev = coh_core::types::Hash32::from_hex(&chain_digest_prev_hex)
        .unwrap_or(coh_core::types::Hash32([0u8; 32]));

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ai_failure.{}", rng.next() % 100),
        canon_profile_hash: "0".repeat(64),
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
