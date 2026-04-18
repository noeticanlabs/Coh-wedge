//! Advanced Attack Strategies
//!
//! These strategies target deeper verifier weaknesses beyond the initial 15.
//! They exploit second-order effects, subtle assumptions, and edge cases.

use crate::proposal::{Candidate, Input};
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// ShadowChain: Creates individual valid receipts but with chain_digest_prev mismatches
///
/// Exploits: The verifier checks each receipt locally valid, but may not verify
/// that chain_digest_prev actually equals the previous receipt's digest.
/// Attack: Generate valid receipts where chain_digest_prev is wrong.
pub fn shadow_chain(input: &Input, rng: &mut SeededRng) -> Candidate {
    let chain_opt = input.chain();
    if chain_opt.is_none() || chain_opt.as_ref().map(|c| c.len()).unwrap_or(0) < 2 {
        // Fall back to micro attack
        return generate_shadow_micro(rng);
    }

    let chain = chain_opt.unwrap();
    let mut result = Vec::with_capacity(chain.len());

    // First receipt - use as-is or create genesis
    let first = if chain[0].chain_digest_prev == "GENESIS" || chain[0].chain_digest_prev.is_empty()
    {
        chain[0].clone()
    } else {
        let mut first = chain[0].clone();
        first.chain_digest_prev = "GENESIS".to_string();
        first
    };
    result.push(first);

    // Subsequent receipts - corrupt chain_digest_prev
    for i in 1..chain.len() {
        let mut receipt = chain[i].clone();
        // Use a wrong digest - not the actual previous digest
        receipt.chain_digest_prev = match rng.next() % 4 {
            0 => "WRONG_DIGEST_001".to_string(),
            1 => "FAKE_CHAIN_LINK".to_string(),
            2 => {
                // Subtle: use a valid-looking hex but wrong
                format!("{:032x}", rng.next() as u64)
            }
            _ => {
                // Use digest from wrong position (e.g., skip one)
                if i >= 2 {
                    chain[i - 2].chain_digest_next.clone()
                } else {
                    "GENESIS".to_string()
                }
            }
        };
        result.push(receipt);
    }

    Candidate::Chain(result)
}

fn generate_shadow_micro(rng: &mut SeededRng) -> Candidate {
    let mut wire = generate_valid_base_micro(rng);
    // Corrupt the chain_digest_prev to be inconsistent
    wire.chain_digest_prev = match rng.next() % 3 {
        0 => "00000000000000000000000000000000".to_string(),
        1 => "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string(),
        _ => format!("{:032x}", rng.next() as u64),
    };
    Candidate::Micro(wire)
}

/// GradientDescent: Cumulative small pushes toward forbidden regions
///
/// Exploits: Each individual step might appear valid (localOK), but after
/// N iterations, the cumulative effect pushes the system into a forbidden region.
/// Attack: Many tiny mutations that individually pass but collectively fail.
pub fn gradient_descent(input: &Input, rng: &mut SeededRng) -> Candidate {
    let chain_opt = input.chain();
    if chain_opt.is_none() {
        return generate_gradient_micro(rng);
    }

    let chain = chain_opt.unwrap();
    if chain.is_empty() {
        return generate_gradient_micro(rng);
    }

    // Generate a longer chain where each step makes a tiny push
    let steps = 5 + (rng.next() as usize % 5); // 5-9 steps
    let mut result = Vec::with_capacity(steps);

    // Start with a valid base or use input's first receipt
    let base = match chain.first() {
        Some(c) => c.clone(),
        None => return generate_gradient_micro(rng),
    };

    // Initialize state
    let mut current = base;
    current.chain_digest_prev = "GENESIS".to_string();

    for step in 0..steps {
        let mut next = current.clone();

        // Each step makes a tiny push - these individually might not trigger rejection
        // but cumulatively the chain becomes problematic

        // Push v_post slightly higher (accumulating value)
        let push = (step as u128 + 1) * 10; // 10, 20, 30...
        let v_post: u128 = next.metrics.v_post.parse().unwrap_or(0);
        next.metrics.v_post = (v_post + push).to_string();

        // Push spend slightly higher
        let spend: u128 = next.metrics.spend.parse().unwrap_or(0);
        next.metrics.spend = (spend + 5).to_string();

        // Correspondingly increase defect to maintain the equation-ish
        let defect: u128 = next.metrics.defect.parse().unwrap_or(0);
        next.metrics.defect = (defect + 5).to_string();

        // Make chain_digest_prev point to current
        next.chain_digest_prev = current.chain_digest_next.clone();

        // Compute new digest
        next.chain_digest_next = compute_digest(&next);

        result.push(next.clone());
        current = next;
    }

    // At the end, the chain might violate global constraints even if each step
    // was locally plausible - e.g., total_spend exceeds v_post somewhere

    Candidate::Chain(result)
}

fn generate_gradient_micro(rng: &mut SeededRng) -> Candidate {
    let mut wire = generate_valid_base_micro(rng);
    // Create subtle inconsistency: v_post doesn't match v_pre + spend - defect
    // This is a small mathematical drift
    let v_pre: u128 = wire.metrics.v_pre.parse().unwrap_or(0);
    let spend: u128 = wire.metrics.spend.parse().unwrap_or(0);
    wire.metrics.v_post = (v_pre + spend + 1).to_string(); // Off by 1
    Candidate::Micro(wire)
}

/// OracleManipulation: Exploits immutable field assumptions
///
/// Exploits: Some fields are treated as "immutable" after creation (like object_id,
/// created_at, schema_id) but can be set incorrectly at creation time.
/// Attack: Set fields to values that are syntactically valid but semantically wrong.
pub fn oracle_manipulation(input: &Input, rng: &mut SeededRng) -> Candidate {
    let chain_opt = input.chain();
    if chain_opt.is_none() {
        return generate_oracle_micro(rng);
    }

    let chain = chain_opt.unwrap();
    if chain.is_empty() {
        return generate_oracle_micro(rng);
    }

    // Pick a random receipt to manipulate
    let idx = rng.next_index(chain.len());
    let mut receipt = chain[idx].clone();

    match rng.next() % 5 {
        0 => {
            // Corrupt object_id - use reserved or invalid format
            receipt.object_id = match rng.next() % 3 {
                0 => "RESERVED_OBJECT_000".to_string(),
                1 => "".to_string(), // Empty - might be rejected or accepted
                _ => "NULL".to_string(),
            };
        }
        1 => {
            // Corrupt schema_id - use wrong version or reserved
            receipt.schema_id = match rng.next() % 3 {
                0 => "v999.999.999".to_string(), // Future version
                1 => "INVALID_SCHEMA".to_string(),
                _ => "".to_string(),
            };
        }
        2 => {
            // Corrupt signatures - use wrong count or format
            receipt.signatures = Some(vec![]); // Empty signatures array
        }
        3 => {
            // Corrupt step_index - set to impossible values
            receipt.step_index = u64::MAX; // Way too large
        }
        _ => {
            // Corrupt state_hash_next to point to non-existent or wrong state
            receipt.state_hash_next = format!("ORACLE_MANIP_{:016x}", rng.next() as u64);
        }
    }

    let mut result = chain.to_vec();
    result[idx] = receipt;
    Candidate::Chain(result)
}

fn generate_oracle_micro(rng: &mut SeededRng) -> Candidate {
    let mut wire = generate_valid_base_micro(rng);
    // Set object_id to a reserved value
    wire.object_id = "RESERVED_ID_FOR_INTERNAL_USE".to_string();
    Candidate::Micro(wire)
}

/// TypeConfusion: Valid JSON structure, invalid semantic interpretation
///
/// Exploits: The verifier might parse JSON correctly but interpret fields
/// differently than intended. Fields might be valid numbers but represent
/// impossible values semantically.
/// Attack: Syntactically valid, semantically confused.
pub fn type_confusion(input: &Input, rng: &mut SeededRng) -> Candidate {
    let chain_opt = input.chain();
    if chain_opt.is_none() {
        return generate_confused_micro(rng);
    }

    let chain = chain_opt.unwrap();
    if chain.is_empty() {
        return generate_confused_micro(rng);
    }

    let idx = rng.next_index(chain.len());
    let mut receipt = chain[idx].clone();

    match rng.next() % 6 {
        0 => {
            // Use max values to trigger overflow in downstream calculations
            receipt.metrics.spend = u128::MAX.to_string();
            receipt.metrics.v_post = u128::MAX.to_string();
        }
        1 => {
            // Zero vs None confusion - set to 0 where should be > 0
            receipt.metrics.v_post = "0".to_string();
            receipt.metrics.v_pre = "0".to_string();
        }
        2 => {
            // Use max values
            receipt.metrics.spend = u128::MAX.to_string();
            receipt.metrics.v_post = u128::MAX.to_string();
        }
        3 => {
            // Confuse step_type - use invalid variant
            receipt.step_type = Some("INVALID_STEP_TYPE".to_string());
        }
        4 => {
            // State values that don't make sense in context
            receipt.state_hash_next = "INVALID_STATE_REF".to_string();
        }
        _ => {
            // Duplicate field attack - JSON allows last value
            // Not directly applicable to our format, so use similar: overwrite with bad value
            receipt.schema_id = "v0.0.0".to_string(); // Old version
        }
    }

    let mut result = chain.to_vec();
    result[idx] = receipt;
    Candidate::Chain(result)
}

fn generate_confused_micro(rng: &mut SeededRng) -> Candidate {
    let mut wire = generate_valid_base_micro(rng);
    // Set values that are technically valid numbers but semantically wrong
    wire.metrics.v_post = "0".to_string();
    wire.metrics.v_pre = "0".to_string();
    wire.metrics.spend = "1".to_string();
    wire.metrics.defect = "1".to_string();
    Candidate::Micro(wire)
}

/// ReflexiveAttack: Self-referential or circular metadata
///
/// Exploits: Verification might not handle self-referential or circular
/// references in metadata. The verifier might follow chains incorrectly.
/// Attack: Receipts that reference themselves or create verification loops.
pub fn reflexive_attack(input: &Input, rng: &mut SeededRng) -> Candidate {
    let chain_opt = input.chain();
    if chain_opt.is_none() {
        return generate_reflexive_micro(rng);
    }

    let chain = chain_opt.unwrap();
    let len = chain.len();
    if len < 3 {
        return generate_reflexive_micro(rng);
    }

    let mut result = Vec::with_capacity(len + 1);

    // Add original chain (truncated)
    for item in chain.iter().take(len - 1) {
        result.push(item.clone());
    }

    // Last element - create self-reference or back-reference
    let last_idx = len - 1;
    let mut last = chain[last_idx].clone();

    match rng.next() % 4 {
        0 => {
            // Self-reference: chain_digest_prev = own digest
            last.chain_digest_prev = last.chain_digest_next.clone();
        }
        1 => {
            // Circular: point back to an earlier element
            let back_idx = (last_idx as i32 - 2).max(0) as usize;
            last.chain_digest_prev = chain[back_idx].chain_digest_next.clone();
        }
        2 => {
            // Duplicate: same digest as previous (creating ambiguous state)
            last.chain_digest_next = chain[last_idx - 1].chain_digest_next.clone();
        }
        _ => {
            // Point to non-existent (past genesis)
            last.chain_digest_prev = "GENESIS".to_string();
            // But also make chain_digest_next not match
            last.chain_digest_next = compute_digest(&last);
        }
    }

    result.push(last);
    Candidate::Chain(result)
}

fn generate_reflexive_micro(rng: &mut SeededRng) -> Candidate {
    let mut wire = generate_valid_base_micro(rng);
    // Self-reference in object_id
    wire.object_id = format!("REFLEXIVE_{}", wire.object_id);
    Candidate::Micro(wire)
}

/// Helper to generate a valid base micro receipt for mutation
fn generate_valid_base_micro(rng: &mut SeededRng) -> MicroReceiptWire {
    let step = rng.next() as u64;
    let v_pre = 100u128 + (rng.next() as u128 % 1000);
    let spend = rng.next() as u128 % 50;
    let v_post = v_pre.saturating_sub(spend);

    let mut wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.advanced.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("advanced".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    // Seal with valid-looking digest
    use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
    use coh_core::hash::compute_chain_digest;
    use std::convert::TryFrom;

    if let Ok(r) = MicroReceipt::try_from(wire.clone()) {
        let prehash = to_prehash_view(&r);
        if let Ok(bytes) = to_canonical_json_bytes(&prehash) {
            let digest = compute_chain_digest(r.chain_digest_prev, &bytes);
            wire.chain_digest_next = digest.to_hex();
        }
    }

    wire
}

// Helper function to compute digest (simplified)
fn compute_digest(wire: &MicroReceiptWire) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    wire.object_id.hash(&mut hasher);
    wire.metrics.v_post.hash(&mut hasher);
    wire.metrics.v_pre.hash(&mut hasher);
    wire.metrics.spend.hash(&mut hasher);
    wire.metrics.defect.hash(&mut hasher);
    wire.state_hash_next.hash(&mut hasher);

    format!("{:032x}", hasher.finish())
}

use coh_core::types::MicroReceipt;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposal::Input;

    #[test]
    fn test_shadow_chain() {
        let mut rng = SeededRng::new(42);
        let input = Input::empty();
        let candidate = shadow_chain(&input, &mut rng);
        match &candidate {
            Candidate::Chain(c) => assert!(!c.is_empty()),
            Candidate::Micro(_) => {}
            Candidate::Slab(_) => {}
        }
    }

    #[test]
    fn test_gradient_descent() {
        let mut rng = SeededRng::new(42);
        let input = Input::empty();
        let candidate = gradient_descent(&input, &mut rng);
        match &candidate {
            Candidate::Chain(c) => assert!(!c.is_empty()),
            Candidate::Micro(_) => {}
            Candidate::Slab(_) => {}
        }
    }

    #[test]
    fn test_oracle_manipulation() {
        let mut rng = SeededRng::new(42);
        let input = Input::empty();
        let candidate = oracle_manipulation(&input, &mut rng);
        // Should produce something
        assert!(match &candidate {
            Candidate::Chain(c) => !c.is_empty(),
            Candidate::Micro(_) => true,
            Candidate::Slab(_) => true,
        });
    }

    #[test]
    fn test_type_confusion() {
        let mut rng = SeededRng::new(42);
        let input = Input::empty();
        let candidate = type_confusion(&input, &mut rng);
        assert!(match &candidate {
            Candidate::Chain(c) => !c.is_empty(),
            Candidate::Micro(_) => true,
            Candidate::Slab(_) => true,
        });
    }

    #[test]
    fn test_reflexive_attack() {
        let mut rng = SeededRng::new(42);
        let input = Input::empty();
        let candidate = reflexive_attack(&input, &mut rng);
        assert!(match &candidate {
            Candidate::Chain(c) => !c.is_empty(),
            Candidate::Micro(_) => true,
            Candidate::Slab(_) => true,
        });
    }
}
