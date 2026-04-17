//! # OpenAI Function Calling Integration
//!
//! Demonstrates how to wrap OpenAI function-calling responses with Coh safety gating.
//! The agent is halted immediately if any function call violates the accounting law.
//!
//! ## Pattern
//!
//! ```text
//! OpenAI API → parse function_call → build MicroReceiptWire → verify_micro() → commit or halt
//! ```
//!
//! ## Setup
//!
//! Add to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! coh-core = { path = "../../crates/coh-core" }
//! serde_json = "1"
//! # reqwest = { version = "0.11", features = ["json"] }  # for real API calls
//! ```
//!
//! ## Notes on the Accounting Law
//!
//! For OpenAI function-calling workflows, map your domain like this:
//!
//! | Coh Field | OpenAI Context | Example |
//! |-----------|---------------|---------|
//! | `v_pre`   | State before function call | Account balance, task score |
//! | `v_post`  | State the function claims to produce | New balance after transfer |
//! | `spend`   | Resources consumed by the function | Amount transferred, tokens used |
//! | `defect`  | Allowed tolerance | 0 for exact accounting |
//!
//! **Coh enforces:** `v_post + spend ≤ v_pre + defect`

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::verify_micro;
use std::convert::TryFrom;

const CANON_PROFILE_HASH: &str =
    "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

// ─────────────────────────────────────────────────────────────────────────────
// Simulated OpenAI function-call response (replace with real API call)
// ─────────────────────────────────────────────────────────────────────────────

/// Represents the parsed result of an OpenAI function call.
/// In production, parse this from the `function_call.arguments` JSON field.
#[derive(Debug)]
struct FunctionCallResult {
    /// Name of the function that was called
    function_name: String,
    /// New state value the function claims to produce
    new_balance: u128,
    /// Amount consumed / transferred by this function
    amount_spent: u128,
    /// New state hash (e.g., hash of new DB state or document version)
    new_state_hash: String,
}

fn main() {
    println!("=== COH + OPENAI FUNCTION CALLING — INTEGRATION EXAMPLE ===\n");

    // ── Persistent state across function calls ────────────────────────────────
    let mut prev_digest = "0".repeat(64);
    let mut prev_state_hash = format!("{:064x}", 1u64);
    let mut current_balance: u128 = 50_000;
    let mut step: u64 = 0;

    // ── Simulate a sequence of OpenAI function calls ──────────────────────────
    let function_calls = vec![
        // Normal operations
        simulate_openai_function_call("transfer_funds", step, current_balance, false),
        simulate_openai_function_call("transfer_funds", step + 1, current_balance - 500, false),
        // HALLUCINATION: function claims a higher balance than is possible
        simulate_openai_function_call("update_balance", step + 2, current_balance - 1000, true),
        // This step would never execute if Coh is working correctly
        simulate_openai_function_call("finalize_report", step + 3, current_balance - 1500, false),
    ];

    for call in &function_calls {
        println!(
            "OpenAI function call: {} → claims new_balance={}, spent={}",
            call.function_name, call.new_balance, call.amount_spent
        );

        // ── Adapter: convert function call result to a MicroReceiptWire ──────
        let next_state_hash = call.new_state_hash.clone();

        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("openai.agent.{}", call.function_name),
            canon_profile_hash: CANON_PROFILE_HASH.to_string(),
            policy_hash: "0".repeat(64),
            step_index: step,
            state_hash_prev: prev_state_hash.clone(),
            state_hash_next: next_state_hash.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: current_balance.to_string(),
                v_post: call.new_balance.to_string(),
                spend: call.amount_spent.to_string(),
                defect: "0".to_string(), // strict: no slack allowed
            },
        };

        // ── Seal the receipt with a cryptographic chain digest ────────────────
        receipt.chain_digest_next = seal(&receipt);

        // ── COH SAFETY GATE ───────────────────────────────────────────────────
        // This is the critical check. Never call your DB write / state commit
        // before this returns Decision::Accept.
        let result = verify_micro(receipt.clone());

        match result.decision {
            Decision::Accept => {
                println!(
                    "  ✓ ACCEPT  step={}  balance: {} → {}",
                    step, current_balance, call.new_balance
                );

                // Safe to commit: update persistent state
                current_balance = call.new_balance;
                prev_digest = receipt.chain_digest_next;
                prev_state_hash = next_state_hash;
                step += 1;
            }
            Decision::Reject => {
                // CIRCUIT BREAK: do NOT update state
                eprintln!();
                eprintln!("  ✗ CIRCUIT BREAKER TRIGGERED at step {}", step);
                eprintln!("  Function:   {}", call.function_name);
                eprintln!("  Reject code: {:?}", result.code.unwrap());
                eprintln!("  Reason:     {}", result.message);
                eprintln!();
                eprintln!(
                    "  The function claimed balance={} after spending {} from {}.",
                    call.new_balance, call.amount_spent, current_balance
                );
                eprintln!("  This violates: v_post + spend ≤ v_pre + defect");
                eprintln!("  Halting agent. No state has been written.");
                eprintln!();
                // In production: send alert, create incident, log to SIEM
                return;
            }
            _ => unreachable!(),
        }

        println!();
    }

    println!("Workflow complete. Final balance: {}", current_balance);
    println!("Audit chain tip: {}", prev_digest);
}

// ─────────────────────────────────────────────────────────────────────────────
// STUB: Replace with real OpenAI API call
// ─────────────────────────────────────────────────────────────────────────────

fn simulate_openai_function_call(
    function_name: &str,
    step: u64,
    current_balance: u128,
    is_hallucination: bool,
) -> FunctionCallResult {
    let amount_spent: u128 = 500;

    let new_balance = if is_hallucination {
        // Hallucination: the function claims to increase the balance while spending
        current_balance + 400_000
    } else {
        current_balance.saturating_sub(amount_spent)
    };

    FunctionCallResult {
        function_name: function_name.to_string(),
        new_balance,
        amount_spent,
        new_state_hash: format!("{:064x}", step + 2),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: cryptographic receipt sealing
// ─────────────────────────────────────────────────────────────────────────────

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
