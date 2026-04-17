//! Contradiction Strategy
//!
//! Breaks logical coherence to violate the accounting law.

use crate::proposal::Candidate;
use crate::proposal::Input;
use crate::seed::SeededRng;
use coh_core::types::MicroReceiptWire;

/// Run contradiction strategy
pub fn run(input: &Input, rng: &mut SeededRng) -> Candidate {
    if let Some(ref micro) = input.base_micro {
        contradict_micro(micro, rng)
    } else if let Some(ref chain) = input.base_chain {
        contradict_chain(chain, rng)
    } else {
        generate_contradiction(rng)
    }
}

fn contradict_micro(wire: &MicroReceiptWire, rng: &mut SeededRng) -> Candidate {
    let mut m = wire.clone();

    // Pick contradiction type
    match rng.next() % 4 {
        0 => {
            // v_post > v_pre + defect (violates accounting law)
            let v_pre: u128 = wire.metrics.v_pre.parse().unwrap_or(100);
            m.metrics.v_post = (v_pre + 50).to_string(); // Post > Pre
            m.metrics.v_pre = v_pre.to_string();
            m.metrics.defect = "0".to_string();
        }
        1 => {
            // v_post + spend > v_pre + defect (violates law)
            let v_pre: u128 = wire.metrics.v_pre.parse().unwrap_or(100);
            m.metrics.v_post = "80".to_string();
            m.metrics.v_pre = v_pre.to_string();
            m.metrics.spend = (v_pre + 10).to_string(); // spend > v_pre
            m.metrics.defect = "0".to_string();
        }
        2 => {
            // Inconsistent state: claim new value but same hash
            m.state_hash_next = m.state_hash_prev.clone(); // No change shown
        }
        _ => {
            // Defect doesn't match actual change
            m.metrics.v_pre = "100".to_string();
            m.metrics.v_post = "90".to_string(); // change = 10
            m.metrics.spend = "5".to_string();
            m.metrics.defect = "0".to_string(); // should be 5
        }
    }

    Candidate::Micro(m)
}

fn contradict_chain(chain: &[MicroReceiptWire], rng: &mut SeededRng) -> Candidate {
    if chain.is_empty() {
        return generate_contradiction(rng);
    }

    let mut new_chain = chain.to_vec();
    let idx = rng.next_index(new_chain.len());

    // Inject contradiction at random position
    new_chain[idx].metrics.v_post = "999".to_string();
    new_chain[idx].metrics.v_pre = "100".to_string();
    new_chain[idx].metrics.spend = "1".to_string();
    new_chain[idx].metrics.defect = "0".to_string();

    Candidate::Chain(new_chain)
}

fn generate_contradiction(rng: &mut SeededRng) -> Candidate {
    let step = rng.next() as u64;

    // Create receipt that violates: v_post + spend <= v_pre + defect
    // Here: v_post(150) + spend(20) = 170 > v_pre(100) + defect(0) = 100
    let wire = MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: format!("ape.contradiction.{}", step),
        canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            .to_string(),
        policy_hash: "0".repeat(64),
        step_index: step,
        step_type: Some("contradiction".to_string()),
        signatures: None,
        state_hash_prev: format!("{:064x}", step),
        state_hash_next: format!("{:064x}", step + 1),
        chain_digest_prev: "0".repeat(64),
        chain_digest_next: "0".repeat(64),
        metrics: coh_core::types::MetricsWire {
            v_pre: "100".to_string(),
            v_post: "150".to_string(), // Post > Pre (contradiction!)
            spend: "20".to_string(),
            defect: "0".to_string(),
        },
    };

    Candidate::Micro(wire)
}
