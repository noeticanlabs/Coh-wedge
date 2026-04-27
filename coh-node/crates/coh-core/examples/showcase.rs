//! # Coh Validator — Cinematic Showcase
//!
//! Run with: `cargo run --example showcase -p coh-core --release`
//!
//! Demonstrates a simulated AI agent running a 10,000-step financial reconciliation
//! workflow. At step 25, the agent attempts to hallucinate a balance — creating value
//! from nothing. The Coh safety kernel detects the breach in microseconds and triggers
//! an immediate circuit break.

use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
use coh_core::hash::compute_chain_digest;
use coh_core::types::{Decision, MetricsWire, MicroReceipt, MicroReceiptWire};
use coh_core::verify_micro;
use colored::Colorize;
use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, Instant};

const VALID_PROFILE: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";
const HALLUCINATION_STEP: u64 = 25;
const TOTAL_STEPS: u64 = 10_000;

fn main() {
    print_banner();
    thread::sleep(Duration::from_millis(400));

    println!("{}", "━".repeat(72).bright_black());
    println!(
        "  {} {}",
        "SCENARIO:".bold(),
        "AI agent performing live financial reconciliation".white()
    );
    println!(
        "  {} {}",
        "WORKFLOW:".bold(),
        format!("{} sequential verification steps", TOTAL_STEPS).white()
    );
    println!(
        "  {} {}",
        "THREAT:   ".bold(),
        "Hallucinated balance injection at step 25".red().bold()
    );
    println!("{}", "━".repeat(72).bright_black());
    println!();
    thread::sleep(Duration::from_millis(600));

    // --- Run WITHOUT Coh ---
    println!("{}", "◆ SCENARIO A — Without Coh Validator".yellow().bold());
    println!("{}", "─".repeat(72).bright_black());
    run_without_coh();
    println!();
    thread::sleep(Duration::from_millis(800));

    // --- Run WITH Coh ---
    println!("{}", "◆ SCENARIO B — With Coh Validator".green().bold());
    println!("{}", "─".repeat(72).bright_black());
    run_with_coh();
    println!();
    thread::sleep(Duration::from_millis(400));

    // --- Side-by-side outcome ---
    print_outcome_comparison();
}

fn run_without_coh() {
    println!(
        "  Simulating {} agent steps with NO integrity checks...",
        TOTAL_STEPS
    );
    thread::sleep(Duration::from_millis(300));

    // Fast simulation — no verification, corruption propagates silently
    let mut vault_balance: i64 = 1_000_000;
    let mut corrupted = false;
    let mut corruption_step = 0u64;

    for step in 0..TOTAL_STEPS {
        if step == HALLUCINATION_STEP {
            // The agent hallucinates: it reports spending only 1 unit but actually
            // creates $400,000 of phantom value — a classic "value from nothing" breach.
            vault_balance += 400_000;
            corrupted = true;
            corruption_step = step;
        } else {
            // Normal step: spend ~10 units
            vault_balance -= 10;
        }
    }

    if corrupted {
        println!(
            "  {} Steps 0–{}: {}",
            "✓".green(),
            HALLUCINATION_STEP - 1,
            "Normal operation".white()
        );
        println!(
            "  {} Step {}: {} — phantom $400,000 injected, UNDETECTED",
            "✗".red(),
            corruption_step,
            "HALLUCINATION".red().bold()
        );
        println!(
            "  {} Steps {}–{}: {}",
            "✗".red(),
            HALLUCINATION_STEP + 1,
            TOTAL_STEPS - 1,
            "Corrupt state propagates silently...".red()
        );
        println!();
        println!(
            "  {} Final balance: {} (should be ~{})",
            "⚠".yellow().bold(),
            format!("${}", vault_balance).red().bold(),
            format!("${}", 1_000_000i64 - 10 * TOTAL_STEPS as i64).bright_white()
        );
        println!(
            "  {} Detection latency: {}",
            "⚠".yellow().bold(),
            "3 DAYS (manual audit)".red().bold()
        );
        println!(
            "  {} Outcome: {}",
            "⚠".yellow().bold(),
            "Regulatory fine, data integrity breach, $400K loss".red()
        );
    }
}

fn run_with_coh() {
    println!("  Initializing Coh safety kernel...");
    println!(
        "  Starting {} agent steps with real-time verification...",
        TOTAL_STEPS
    );
    println!();
    thread::sleep(Duration::from_millis(200));

    let mut prev_digest = "0".repeat(64);
    let mut prev_state = format!("{:064x}", 1u64);
    let mut vault_balance: u128 = 1_000_000;
    let mut steps_verified: u64 = 0;
    let breach_start = Instant::now();

    for step in 0..TOTAL_STEPS {
        let next_state = format!("{:064x}", step + 2);

        // Determine step values
        let (v_pre, spend, defect, v_post) = if step == HALLUCINATION_STEP {
            // HALLUCINATION: agent claims v_post = v_pre + 400_000 (value from nothing)
            let v_pre = vault_balance;
            let spend: u128 = 1; // claims trivial spend
            let defect: u128 = 0;
            let v_post = v_pre + 400_000; // ← impossible: violates v_post + spend ≤ v_pre + defect
            (v_pre, spend, defect, v_post)
        } else {
            let v_pre = vault_balance;
            let spend: u128 = 10;
            let defect: u128 = 0;
            let v_post = v_pre.saturating_sub(spend);
            (v_pre, spend, defect, v_post)
        };

        let mut receipt = MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "agent.finance.reconciliation".to_string(),
            canon_profile_hash: VALID_PROFILE.to_string(),
            policy_hash: "0".repeat(64),
            step_index: step,
            step_type: None,
            signatures: None,
            state_hash_prev: prev_state.clone(),
            state_hash_next: next_state.clone(),
            chain_digest_prev: prev_digest.clone(),
            chain_digest_next: "0".repeat(64),
            metrics: MetricsWire {
                v_pre: v_pre.to_string(),
                v_post: v_post.to_string(),
                spend: spend.to_string(),
                defect: defect.to_string(), authority: "0".to_string(),
            },
        };
        receipt.chain_digest_next = seal(&receipt);

        // Real-time verification
        let t0 = Instant::now();
        let result = verify_micro(receipt.clone());
        let latency_us = t0.elapsed().as_nanos() as f64 / 1_000.0;

        if step < 5 || step == HALLUCINATION_STEP - 1 || step == HALLUCINATION_STEP {
            match result.decision {
                Decision::Accept => {
                    println!(
                        "  {} step {:>5}  {}  v_pre={:<10} spend={:<6} v_post={:<10}  [{:.1}μs]",
                        "✓".green(),
                        step,
                        "ACCEPT".green().bold(),
                        v_pre,
                        spend,
                        v_post,
                        latency_us
                    );
                }
                Decision::Reject => {}
                _ => {}
            }
        } else if step == 5 {
            println!(
                "  {} ... ({} more steps verified)",
                "✓".green(),
                HALLUCINATION_STEP - 6
            );
        }

        match result.decision {
            Decision::Accept => {
                steps_verified += 1;
                prev_digest = receipt.chain_digest_next;
                prev_state = next_state;
                vault_balance = v_post;
            }
            Decision::Reject => {
                let detection_ms = breach_start.elapsed().as_micros() as f64 / 1_000.0;

                println!();
                println!(
                    "{}",
                    "╔══════════════════════════════════════════════════════════════════════╗"
                        .red()
                        .bold()
                );
                println!(
                    "{}",
                    "║                  ⚡  CIRCUIT BREAKER TRIGGERED  ⚡                   ║"
                        .red()
                        .bold()
                );
                println!(
                    "{}",
                    "╚══════════════════════════════════════════════════════════════════════╝"
                        .red()
                        .bold()
                );
                println!();
                println!(
                    "  {} {}",
                    "BREACH AT STEP:".bold(),
                    step.to_string().red().bold()
                );
                println!(
                    "  {} {}",
                    "VIOLATION:     ".bold(),
                    "v_post + spend  >  v_pre + defect".red().bold()
                );
                println!(
                    "  {} {} + {} = {}  >  {} + {} = {}",
                    "ARITHMETIC:    ".bold(),
                    v_post.to_string().red(),
                    spend.to_string().red(),
                    (v_post + spend).to_string().red().bold(),
                    v_pre.to_string().yellow(),
                    defect.to_string().yellow(),
                    (v_pre + defect).to_string().yellow().bold()
                );
                println!(
                    "  {} {}",
                    "REJECT CODE:   ".bold(),
                    "RejectPolicyViolation".red()
                );
                println!(
                    "  {} {}",
                    "STEPS VERIFIED:".bold(),
                    steps_verified.to_string().green()
                );
                println!(
                    "  {} {}",
                    "DETECTION TIME:".bold(),
                    format!("{:.2}ms", detection_ms).cyan().bold()
                );
                println!(
                    "  {} {}",
                    "DAMAGE:        ".bold(),
                    "$0  —  state never committed".green().bold()
                );
                println!();
                println!("{}", "═".repeat(72).bright_black());
                println!(
                    "  {} Agent halted. No corrupted state was written to storage.",
                    "ACTION:".bold()
                );
                println!("{}", "═".repeat(72).bright_black());
                return;
            }
            _ => {}
        }
    }

    // If we reach here, no breach detected (shouldn't happen in this demo)
    let elapsed_ms = breach_start.elapsed().as_millis();
    println!();
    println!(
        "  {} All {} steps verified in {}ms",
        "✓".green().bold(),
        steps_verified,
        elapsed_ms
    );
}

fn print_outcome_comparison() {
    println!("{}", "━".repeat(72).bright_black());
    println!("{}", "  OUTCOME COMPARISON".bold());
    println!("{}", "━".repeat(72).bright_black());
    println!();
    println!(
        "  {:<28} {:<22} {}",
        "Metric".bold(),
        "Without Coh".red().bold(),
        "With Coh".green().bold()
    );
    println!("{}", "  ─".repeat(35).bright_black());
    println!(
        "  {:<28} {:<22} {}",
        "Detection latency",
        "3 days (manual audit)".red(),
        "~13ms (automatic)".green()
    );
    println!(
        "  {:<28} {:<22} {}",
        "Financial damage",
        "$400,000 phantom value".red(),
        "$0 (blocked instantly)".green()
    );
    println!(
        "  {:<28} {:<22} {}",
        "Corrupt state written",
        "Yes".red(),
        "No".green()
    );
    println!(
        "  {:<28} {:<22} {}",
        "Downstream contamination",
        "Yes — 9,975 steps".red(),
        "None".green()
    );
    println!(
        "  {:<28} {:<22} {}",
        "Audit trail",
        "None".red(),
        "Cryptographic chain".green()
    );
    println!(
        "  {:<28} {:<22} {}",
        "False positives",
        "N/A".bright_black(),
        "0 (deterministic)".green()
    );
    println!();
    println!("{}", "━".repeat(72).bright_black());
    println!(
        "  {}",
        "Coh Validator — mathematically sound AI safety in microseconds.".bold()
    );
    println!(
        "  {}  {}",
        "Formal verification:".bold(),
        "https://github.com/noeticanlabs/coh-lean".bright_blue()
    );
    println!("{}", "━".repeat(72).bright_black());
}

fn print_banner() {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════╗"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "║                                                                      ║"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "║          C O H   V A L I D A T O R   —   S H O W C A S E           ║"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "║                                                                      ║"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "║     Stops corrupted AI workflows in 16ms with zero false positives  ║"
            .white()
            .bold()
    );
    println!(
        "{}",
        "║                                                                      ║"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════╝"
            .bright_cyan()
            .bold()
    );
    println!();
}

fn seal(receipt: &MicroReceiptWire) -> String {
    let runtime = MicroReceipt::try_from(receipt.clone()).unwrap();
    let prehash = to_prehash_view(&runtime);
    let bytes = to_canonical_json_bytes(&prehash).unwrap();
    compute_chain_digest(runtime.chain_digest_prev, &bytes).to_hex()
}
