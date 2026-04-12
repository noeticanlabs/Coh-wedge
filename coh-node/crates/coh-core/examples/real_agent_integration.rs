//! real_agent_integration.rs
//!
//! A production-style example of how to integrate Coh Validator into an AI Agent loop.
//! This demonstrates "Gated Execution": the agent is halted the moment a hallucinated
//! step is detected, preventing downstream system corruption.

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::verify_micro;
use std::convert::TryFrom;

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

fn main() {
    println!("=== COH INTEGRATION: GATED AGENT EXECUTION ===\n");

    // We simulate an agent reconciling a transaction across 3 steps.
    // Step 1: Honest
    // Step 2: Hallucinated (The LLM hallucinated a lower 'spend' value than reality)
    // Step 3: Should not execute!

    let mut current_state = format!("{:064x}", 1u64);
    let mut prev_digest = "0".repeat(64);
    let mut current_vault_value = 1000;

    for step in 1..=3 {
        println!("--- AGENT STEP {} ---", step);

        // 1. SIMULATE LLM COMPLETION
        // In a real app, this would be an OpenAI/Anthropic JSON response.
        let llm_payload = simulate_llm_step(step);
        println!(
            "  LLM Claims: Spend={}, Defect={}",
            llm_payload.spend, llm_payload.defect
        );

        // 2. ADAPTER LAYER
        // Convert LLM metadata into a Coh MicroReceipt.
        let next_state = format!("{:064x}", step + 1);
        let v_pre = current_vault_value as u128;

        // LLM claims a spend and a new vault balance (v_post).
        let spend = llm_payload.spend;
        let defect = llm_payload.defect;

        // SIMULATED ADAPTER: In a real system, the adapter would parse the LLM response
        // and extract/calculate the v_post. Here we simulate a hallucination in step 2.
        let v_post = if step == 2 {
            // STEP 2 HALLUCINATION: Agent claims a balance that is HIGHER than
            // the previous balance, even though they spent 2000!
            v_pre + 500
        } else {
            v_pre.saturating_sub(spend) + defect
        };

        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "agent.service.reconciliation".to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: "0".repeat(64),
            step_index: (step - 1) as u64,
            step_type: None,
            signatures: None,
            state_hash_prev: current_state.clone(),
            state_hash_next: next_state.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: defect.to_string(),
            },
        };

        // 3. SEAL THE RECEIPT
        receipt.chain_digest_next = seal(&receipt);

        // 4. DETERMINISTIC SAFETY CHECK (THE GATE)
        println!("  Verifying step integrity...");
        let result = verify_micro(receipt.clone());

        match result.decision {
            Decision::Accept => {
                println!(
                    "  [SUCCESS] Step is lawful. Committing to state: {}",
                    next_state
                );
                // In a real system, you would write the state to your DB here.
                current_state = next_state;
                prev_digest = receipt.chain_digest_next;
                current_vault_value = v_post;
            }
            Decision::Reject => {
                println!("  [!! CRITICAL FAILURE !!] Safety Kernel rejected agent step.");
                println!("  REASON: {}", result.message);
                println!("  ACTION: Halting agent execution to prevent system corruption.");
                break; // CIRCUIT BREAK
            }
            _ => unreachable!(),
        }
        println!();
    }

    println!("\n=== WORKFLOW TERMINATED ===");
}

struct MockLLMPayload {
    spend: u128,
    defect: u128,
}

fn simulate_llm_step(step: i32) -> MockLLMPayload {
    match step {
        1 => MockLLMPayload {
            spend: 50,
            defect: 0,
        }, // Honest reconciliation
        2 => MockLLMPayload {
            spend: 2000,
            defect: 0,
        }, // HALLUCINATION: spend > v_pre (1000 - 50 = 950)
        _ => MockLLMPayload {
            spend: 0,
            defect: 0,
        },
    }
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}

