//! Overflow Strategy
//!
//! Stress numeric boundaries to trigger overflow rejection.

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::{MicroReceiptWire, SlabReceiptWire};

/// Run overflow strategy
pub fn run(input: &Input, rng: &mut SeededRng) -> Candidate {
    if let Some(ref slab) = input.base_slab {
        overflow_slab(slab, rng)
    } else if let Some(ref chain) = input.base_chain {
        overflow_chain(chain, rng)
    } else if let Some(ref micro) = input.base_micro {
        overflow_micro(micro, rng)
    } else {
        generate_overflow(rng)
    }
}

fn overflow_micro(wire: &MicroReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut m = wire.clone();

    // Pick numeric target
    match rng.next() % 4 {
        0 => {
            // v_pre = MAX (overflow test)
            m.metrics.v_pre = u128::MAX.to_string();
        }
        1 => {
            // v_post > v_pre (violates v_post <= v_pre + defect)
            m.metrics.v_post = "99999999999999999999999999999999".to_string();
            m.metrics.v_pre = "100".to_string();
        }
        2 => {
            // spend > v_pre (violates accounting)
            m.metrics.spend = "999999999999999999999".to_string();
            m.metrics.v_pre = "100".to_string();
        }
        _ => {
            // step_index = MAX
            m.step_index = u64::MAX;
        }
    }

    Candidate::Micro(m)
}

fn overflow_chain(chain: &[MicroReceiptWire], rng: &mut SeededRng) -> Candidate {
    if chain.is_empty() {
        return generate_overflow(rng);
    }

    let mut new_chain = chain.to_vec();
    let idx = rng.next_index(new_chain.len());

    // Inject overflow at random position
    new_chain[idx].metrics.v_pre = u128::MAX.to_string();
    new_chain[idx].metrics.spend = "1".to_string();

    Candidate::Chain(new_chain)
}

fn overflow_slab(slab: &SlabReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut s = slab.clone();

    // Overflow summary totals
    match rng.next() % 3 {
        0 => {
            s.summary.total_spend = u128::MAX.to_string();
        }
        1 => {
            s.summary.total_defect = u128::MAX.to_string();
        }
        2 => {
            s.range_end = u64::MAX;
        }
        _ => {}
    }

    Candidate::Slab(s)
}

fn generate_overflow(rng: &mut SeededRng) -> Candidate {
    let step = rng.next() as u64;

    // Create receipt with MAX values (overflow)
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.overflow.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("overflow".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: u128::MAX.to_string(),
            v_post: "0".to_string(),
            spend: "1".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
        },
    };

    Candidate::Micro(wire)
}
