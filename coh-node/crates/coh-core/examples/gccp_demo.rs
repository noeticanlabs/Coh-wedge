use coh_core::types::{MicroReceiptWire, Decision, MicroReceipt};
use coh_core::verify_micro::verify_micro;
use coh_time::CohTimeEngine;
use coh_gccp::{GccpState, GccpVerifier};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    println!("=== GCCP + Coh-Time Integrated Demo ===");

    // 1. Initialize Engines
    let mut time_engine = CohTimeEngine::new();
    let gccp_verifier = GccpVerifier::default();
    let mut gccp_state = GccpState::default();

    println!("Initial State: Attempt={}, Accepted={}", 
        time_engine.state().attempt_index, 
        time_engine.state().accepted_index
    );

    // 2. Load Valid GCCP Vectors
    let valid_path = "vectors/gccp/valid_gccp_power.jsonl";
    println!("\n--- Processing Valid Transitions from {} ---", valid_path);
    process_vector_file(valid_path, &mut time_engine, &gccp_verifier, &mut gccp_state);

    // 3. Load Reject GCCP Vectors
    let reject_path = "vectors/gccp/reject_gccp_power_breach.jsonl";
    println!("\n--- Processing Reject Transitions from {} ---", reject_path);
    
    // Simulate a thermal breach for the reject demo
    gccp_state.thermal.die_temp = 95.0; 
    process_vector_file(reject_path, &mut time_engine, &gccp_verifier, &mut gccp_state);

    // 4. Final Summary
    println!("\n=== Demo Summary ===");
    println!("Final Indices: Attempt={}, Accepted={}", 
        time_engine.state().attempt_index, 
        time_engine.state().accepted_index
    );
    println!("Ledger Entries: {}", time_engine.get_ledger().len());
    println!("Attempt Log Entries: {}", time_engine.get_attempt_log().len());
}

fn fix_hash(h: String) -> String {
    if h.len() == 64 {
        h
    } else if h.len() < 64 {
        format!("{:0<64}", h)
    } else {
        h[..64].to_string()
    }
}

fn process_vector_file(
    path: &str, 
    time_engine: &mut CohTimeEngine, 
    gccp_verifier: &GccpVerifier,
    gccp_state: &mut GccpState
) {
    // Try multiple relative paths depending on where it's run from
    let attempts = vec![
        path.to_string(),
        format!("../../../{}", path), // From coh-core/examples
        format!("../../{}", path),    // From coh-node
        format!("./{}", path),         // From root
        format!("coh-node/{}", path),  // From root
    ];
    
    let mut opened_file = None;
    for attempt in attempts {
        if let Ok(f) = File::open(&attempt) {
            opened_file = Some(f);
            break;
        }
    }
    
    let file = match opened_file {
        Some(f) => f,
        None => {
            println!("Error: Could not find {} in any expected location.", path);
            return;
        }
    };

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let mut wire: MicroReceiptWire = serde_json::from_str(&line).unwrap();
        
        // Fix short hashes in vectors
        wire.state_hash_prev = fix_hash(wire.state_hash_prev);
        wire.state_hash_next = fix_hash(wire.state_hash_next);
        wire.chain_digest_prev = fix_hash(wire.chain_digest_prev);
        wire.chain_digest_next = fix_hash(wire.chain_digest_next);
        wire.canon_profile_hash = fix_hash(wire.canon_profile_hash);
        wire.policy_hash = fix_hash(wire.policy_hash);

        let receipt = MicroReceipt::try_from(wire.clone()).unwrap();
        let digest = receipt.chain_digest_next.clone();

        // Step A: Core Verification
        // Note: verify_micro will likely reject due to digest mismatch because we modified the hashes,
        // but for this demo we are interested in the integration flow.
        let core_res = verify_micro(wire);
        
        // Step B: GCCP Verification
        let gccp_res = gccp_verifier.verify_transition(gccp_state, &receipt);

        let final_decision = if core_res.decision == Decision::Accept && gccp_res.is_ok() {
            Decision::Accept
        } else {
            // For the demo, we might want to accept if core_res failed ONLY due to RejectChainDigest
            // but for a strict demo we follow the decision.
            Decision::Reject
        };

        let error_code = if let Err(e) = gccp_res {
            Some(e)
        } else {
            core_res.code
        };

        // Step C: Apply to Time Engine
        let (att, acc) = time_engine.apply_decision(
            digest, 
            final_decision, 
            error_code, 
            if final_decision == Decision::Accept { Some(receipt.state_hash_next) } else { None }
        );

        println!("Step {}: Core={:?}, GCCP={:?} -> Decision={:?} [Att={}, Acc={}]", 
            receipt.step_index,
            core_res.decision,
            if gccp_res.is_ok() { "OK" } else { "BREACH" },
            final_decision,
            att,
            acc
        );
    }
}
