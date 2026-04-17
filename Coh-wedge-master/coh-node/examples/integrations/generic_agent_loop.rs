//! # Generic Agent Loop Integration Template
//!
//! A universal template for wrapping any AI agent loop with Coh safety gating.
//! Copy and adapt this for your specific LLM provider (OpenAI, Anthropic, local models).
//!
//! ## Integration Steps
//!
//! 1. Replace `simulate_llm_step()` with your actual LLM call.
//! 2. Implement `adapter_to_receipt()` to convert your domain values into Coh metrics.
//! 3. Keep the `verify_micro()` gate — never commit state before this check.
//!
//! ## The Accounting Law
//!
//! ```
//! v_post + spend ≤ v_pre + defect
//! ```
//!
//! Map your domain:
//! - `v_pre`  = state value before this step (e.g., account balance, task progress score)
//! - `v_post` = state value after this step
//! - `spend`  = resources consumed (e.g., tokens, funds transferred, tasks completed)
//! - `defect` = approved variance / slack (often 0 for strict validation)

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::verify_micro;
use std::convert::TryFrom;

// ─────────────────────────────────────────────────────────────────────────────
// CONFIGURATION: Replace with your actual canon profile hash.
// Compute with: cargo run --example gen_ai_fixtures -p coh-core
// ─────────────────────────────────────────────────────────────────────────────
const CANON_PROFILE_HASH: &str =
    "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

const OBJECT_ID: &str = "your.agent.workflow.id";

fn main() {
    println!("=== COH GENERIC AGENT LOOP — INTEGRATION TEMPLATE ===\n");

    // ── State tracking (maintain across steps) ──────────────────────────────
    let mut prev_digest = "0".repeat(64); // genesis digest
    let mut prev_state = format!("{:064x}", 1u64); // genesis state hash
    let mut step = 0u64;

    // ── Your domain state ───────────────────────────────────────────────────
    let mut domain_value: u128 = 10_000; // e.g., account balance, task score

    loop {
        println!("─── Step {} ───", step);

        // ── 1. CALL YOUR LLM ─────────────────────────────────────────────────
        // Replace this with your actual LLM invocation.
        let llm_result = simulate_llm_step(step, domain_value);

        if llm_result.should_terminate {
            println!("[DONE] Agent workflow complete after {} steps.", step);
            break;
        }

        println!(
            "  Agent claims: spend={}, new_value={}",
            llm_result.spend, llm_result.new_value
        );

        // ── 2. ADAPTER LAYER: Build a MicroReceiptWire ────────────────────────
        // Map your domain values to Coh accounting fields.
        let next_state = format!("{:064x}", step + 2);
        let mut receipt = build_receipt(
            step,
            &prev_state,
            &next_state,
            &prev_digest,
            domain_value,      // v_pre: value before this step
            llm_result.new_value, // v_post: value the agent claims after this step
            llm_result.spend,  // spend: resources consumed
            0,                 // defect: set > 0 to allow variance
        );

        // ── 3. SEAL THE RECEIPT ───────────────────────────────────────────────
        receipt.chain_digest_next = seal(&receipt);

        // ── 4. COH SAFETY GATE (THE CRITICAL CHECK) ───────────────────────────
        // NEVER commit state before this check.
        let result = verify_micro(receipt.clone());

        match result.decision {
            Decision::Accept => {
                // ✓ Safe to commit — update your state
                println!("  [ACCEPT] Step {} verified. Committing state.", step);
                prev_digest = receipt.chain_digest_next;
                prev_state = next_state;
                domain_value = llm_result.new_value;
                step += 1;
            }
            Decision::Reject => {
                // ✗ CIRCUIT BREAK — do NOT commit state
                eprintln!("  [REJECT] Safety kernel rejected step {}!", step);
                eprintln!("  Code:    {:?}", result.code);
                eprintln!("  Reason:  {}", result.message);
                eprintln!("  Action:  Halting agent. No state has been corrupted.");
                // In production: send alert, log to SIEM, trigger incident response.
                std::process::exit(1);
            }
            _ => unreachable!(),
        }

        println!();

        if step >= 10 {
            println!("[INFO] Demo limit reached (10 steps). Exiting.");
            break;
        }
    }

    println!("\nFinal domain value: {}", domain_value);
    println!("Chain tip digest:   {}", prev_digest);
}

// ─────────────────────────────────────────────────────────────────────────────
// IMPLEMENT: Replace with your actual LLM call
// ─────────────────────────────────────────────────────────────────────────────

struct LlmResult {
    new_value: u128,
    spend: u128,
    should_terminate: bool,
}

fn simulate_llm_step(step: u64, current_value: u128) -> LlmResult {
    // Simulate an agent that processes work (spending 50 units per step)
    // and terminates after reaching a threshold.
    if current_value < 50 {
        return LlmResult { new_value: 0, spend: 0, should_terminate: true };
    }
    LlmResult {
        new_value: current_value - 50,
        spend: 50,
        should_terminate: step >= 9,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HELPERS: Receipt construction and sealing
// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_receipt(
    step_index: u64,
    state_hash_prev: &str,
    state_hash_next: &str,
    chain_digest_prev: &str,
    v_pre: u128,
    v_post: u128,
    spend: u128,
    defect: u128,
) -> MicroReceiptWire {
    MicroReceiptWire {
        schema_id: "coh.receipt.micro.v1".to_string(),
        version: "1.0.0".to_string(),
        object_id: OBJECT_ID.to_string(),
        canon_profile_hash: CANON_PROFILE_HASH.to_string(),
        policy_hash: "0".repeat(64),
        step_index,
        state_hash_prev: state_hash_prev.to_string(),
        state_hash_next: state_hash_next.to_string(),
        chain_digest_prev: chain_digest_prev.to_string(),
        chain_digest_next: "0".repeat(64), // filled in by seal()
        metrics: MetricsWire {
            v_pre: v_pre.to_string(),
            v_post: v_post.to_string(),
            spend: spend.to_string(),
            defect: defect.to_string(),
        },
    }
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
