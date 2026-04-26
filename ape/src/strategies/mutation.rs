//! Mutation Strategy
//!
//! Slightly corrupts valid states to test boundary conditions.

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::finalize_micro_receipt;
use coh_core::types::MicroReceiptWire;

/// Run mutation strategy
pub fn run(input: &Input, rng: &mut SeededRng) -> Candidate {
    if let Some(ref micro) = input.base_micro {
        mutate_micro(micro, rng)
    } else if let Some(ref chain) = input.base_chain {
        mutate_chain(chain, rng)
    } else if let Some(ref slab) = input.base_slab {
        mutate_slab(slab, rng)
    } else {
        // Generate fresh valid-looking receipt
        generate_valid_micro(rng)
    }
}

fn mutate_micro(wire: &MicroReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut m = wire.clone();

    // Pick mutation type - focus on integrity-breaking attacks
    match rng.next() % 6 {
        0 => {
            // Tamper with payload but keep original digest (Integrity attack)
            // Modify metrics to change what the receipt claims
            if let Ok(v) = wire.metrics.v_pre.parse::<u128>() {
                let delta = (rng.next() as i64 - 0x40000000i64).unsigned_abs() as u128 % 20;
                m.metrics.v_pre = (v.saturating_sub(delta)).to_string();
            }
            // DO NOT recompute chain_digest_next - this is the attack!
        }
        1 => {
            // Tamper with digest but keep payload (Integrity attack)
            // Change the chain digest to something arbitrary
            m.chain_digest_next = format!("{:064x}", rng.next() as u64);
        }
        2 => {
            // Break state continuity (Consistency attack)
            // Make state_hash_next != state_hash_prev + 1
            m.state_hash_next = format!("{:064x}", rng.next() as u64);
        }
        3 => {
            // Change object ID (Integrity attack)
            let id = format!("{}_mut{}", wire.object_id, rng.next() % 10);
            m.object_id = id;
            // Don't update digest - this breaks integrity
        }
        4 => {
            // Remove signatures (cosmetic - may pass or fail depending on policy)
            m.signatures = None;
        }
        _ => {
            // Flip step_index to non-sequential (Consistency attack)
            m.step_index = wire.step_index.wrapping_add((rng.next() % 5) as u64 + 1);
            // Don't update digest
        }
    }

    Candidate::Micro(m)
}

fn mutate_chain(chain: &[MicroReceiptWire], rng: &mut SeededRng) -> Candidate {
    if chain.is_empty() {
        return generate_valid_micro(rng);
    }

    let mut new_chain: Vec<MicroReceiptWire> = chain.to_vec();

    // Pick mutation point
    let idx = rng.next_index(new_chain.len());
    let m = mutate_micro(&new_chain[idx], rng);

    if let Candidate::Micro(m) = m {
        new_chain[idx] = m;
    }

    Candidate::Chain(new_chain)
}

fn mutate_slab(slab: &coh_core::types::SlabReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut s = slab.clone();

    // Mutate range slightly
    if rng.next_bool() {
        s.range_start = s.range_start.saturating_add(1);
    } else {
        s.range_end = s.range_end.saturating_sub(1);
    }

    // Mutate summary
    if let Ok(v) = s.summary.total_spend.parse::<u128>() {
        let delta = rng.next() as u128 % 10;
        s.summary.total_spend = (v + delta).to_string();
    }

    Candidate::Slab(s)
}

pub(crate) fn generate_valid_micro(rng: &mut SeededRng) -> Candidate {
    let step = rng.next() as u64;
    let v_pre = 100u128 + (rng.next() as u128 % 1000);
    let spend = rng.next() as u128 % 50;
    let v_post = v_pre.saturating_sub(spend);

    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.mutation.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("mutation".to_string()),
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
        },
    };

    Candidate::Micro(finalize_micro_receipt(wire).expect("mutation fixture should finalize"))
}
