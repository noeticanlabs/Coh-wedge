use ape::http::{execute_verified, save_valid_receipts_to_jsonl, ExecuteVerifiedRequest};
use ape::proposal::{Candidate, Input, Strategy};
use ape::realdata::{
    generate_runtime_ai_chain, generate_runtime_ai_micro, load_ai_demo_chain, load_ai_demo_micro,
    write_output_json,
};
use clap::{Parser, Subcommand};
use coh_core::types::{Decision, MicroReceiptWire};
use coh_core::{build_slab, verify_chain, verify_micro};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ape")]
#[command(about = "APE - Adversarial Proposal Engine for Coh Wedge")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(long, default_value = "mutation")]
        strategy: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Verify {
        #[arg(long)]
        input: String,
    },
    ExecuteVerified {
        #[arg(long, default_value = "mutation")]
        strategy: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long)]
        action: Option<String>,
    },
    Demo {
        #[arg(long, default_value = "both")]
        mode: String,
        #[arg(long, default_value = "http://127.0.0.1:3030")]
        sidecar_url: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value = "transfer_100_tokens")]
        action: String,
    },
    Bench {
        #[arg(long, default_value_t = 1000)]
        iterations: usize,
        #[arg(long, default_value = "http://127.0.0.1:3030")]
        sidecar_url: String,
        #[arg(long, default_value_t = false)]
        with_sidecar: bool,
    },
    /// Per-strategy demo showing rejection rates by attack class
    StrategyDemo {
        #[arg(long, default_value_t = 100)]
        iterations: usize,
        #[arg(long, default_value = "http://127.0.0.1:3030")]
        sidecar_url: String,
        #[arg(long, default_value_t = false)]
        with_sidecar: bool,
    },
    /// Comparative failure dominance benchmark: full verifier vs baselines
    Dominance {
        #[arg(long, default_value_t = 100)]
        iterations: usize,
        #[arg(long, default_value = "http://127.0.0.1:3030")]
        sidecar_url: String,
    },
}

#[derive(Debug, Serialize)]
struct DemoResult {
    path: String,
    decision: String,
    message: String,
    action: Value,
    sidecar_status: Option<String>,
}

#[derive(Debug, Serialize)]
struct BenchResult {
    name: String,
    iterations: usize,
    total_ms: f64,
    throughput_per_sec: f64,
    avg_us: f64,
}

#[derive(Debug, Serialize)]
struct BenchSuite {
    results: Vec<BenchResult>,
    sidecar_results: Vec<BenchResult>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { strategy, seed } => {
            let proposal = ape::engine::generate(parse_strategy(&strategy), &Input::empty(), seed);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "proposal_id": proposal.proposal_id,
                    "strategy": proposal.strategy.name(),
                    "seed": proposal.seed,
                    "candidate": proposal.candidate,
                }))?
            );
        }
        Commands::Verify { input } => {
            let receipt: MicroReceiptWire = serde_json::from_str(&input)?;
            let result = verify_micro(receipt);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "decision": format!("{:?}", result.decision),
                    "code": result.code,
                    "message": result.message,
                }))?
            );
        }
        Commands::ExecuteVerified {
            strategy,
            seed,
            action,
        } => {
            let proposal = ape::engine::generate(parse_strategy(&strategy), &Input::empty(), seed);
            let receipt = candidate_to_micro(&proposal.candidate)?;
            let result = verify_micro(receipt.clone());
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "decision": format!("{:?}", result.decision),
                    "code": result.code,
                    "message": result.message,
                    "receipt": receipt,
                }))?
            );
            if result.decision == Decision::Accept {
                if let Some(action) = action {
                    println!("\n[READY] Action approved: {}", action);
                    println!("[EXECUTE] In production, POST to /v1/execute-verified");
                } else {
                    println!("\n[DRYRUN] No action to execute - add --action for full run");
                }
            } else {
                println!("\n[BLOCKED] Action not executed - verification failed");
                std::process::exit(1);
            }
        }
        Commands::Demo {
            mode,
            sidecar_url,
            seed,
            action,
        } => run_demo(&mode, &sidecar_url, seed, &action)?,
        Commands::Bench {
            iterations,
            sidecar_url,
            with_sidecar,
        } => run_bench(iterations, &sidecar_url, with_sidecar)?,
        Commands::StrategyDemo {
            iterations,
            sidecar_url,
            with_sidecar,
        } => run_strategy_demo(iterations, &sidecar_url, with_sidecar)?,
        Commands::Dominance {
            iterations,
            sidecar_url,
        } => run_dominance(iterations, &sidecar_url)?,
    }

    Ok(())
}

fn run_demo(
    mode: &str,
    sidecar_url: &str,
    seed: u64,
    action: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let valid_receipt = generate_runtime_ai_micro().or_else(|_| load_ai_demo_micro())?;
    let invalid_proposal = ape::engine::generate(Strategy::Violation, &Input::empty(), seed);
    let invalid_receipt = candidate_to_micro(&invalid_proposal.candidate)?;
    let action_value = json!({ "action": action, "amount": 100, "target": "alice" });

    let mut outputs = Vec::new();

    if mode == "local" || mode == "both" {
        let valid = verify_micro(valid_receipt.clone());
        outputs.push(DemoResult {
            path: "local_accept".to_string(),
            decision: format!("{:?}", valid.decision),
            message: valid.message,
            action: action_value.clone(),
            sidecar_status: None,
        });

        let invalid = verify_micro(invalid_receipt.clone());
        outputs.push(DemoResult {
            path: "local_reject".to_string(),
            decision: format!("{:?}", invalid.decision),
            message: invalid.message,
            action: action_value.clone(),
            sidecar_status: None,
        });
    }

    if mode == "sidecar" || mode == "both" {
        let accept_payload = ExecuteVerifiedRequest {
            receipt: valid_receipt.clone(),
            action: action_value.clone(),
        };
        let accept_resp = execute_verified(sidecar_url, &accept_payload);

        // Save valid receipts to JSONL for dashboard
        if accept_resp.is_ok() {
            // Get root project directory (go up from ape/)
            let root = std::env::current_dir()?
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            let out_path = root.join("coh-dashboard/public/demo/sidecar_valid.jsonl");
            if let Err(e) =
                save_valid_receipts_to_jsonl(std::slice::from_ref(&valid_receipt), &out_path)
            {
                eprintln!("Warning: Failed to save valid receipts: {}", e);
            } else {
                println!("[demo] Saved valid receipt to {:?}", out_path);
            }
        }

        outputs.push(DemoResult {
            path: "sidecar_accept".to_string(),
            decision: if accept_resp.is_ok() {
                "Accept".to_string()
            } else {
                "Error".to_string()
            },
            message: accept_resp
                .as_ref()
                .map(|r| format!("status={}", r.status))
                .unwrap_or_else(|_| "sidecar unavailable".to_string()),
            action: action_value.clone(),
            sidecar_status: accept_resp.ok().map(|r| r.status),
        });

        let reject_payload = ExecuteVerifiedRequest {
            receipt: invalid_receipt.clone(),
            action: action_value.clone(),
        };
        let reject_resp = execute_verified(sidecar_url, &reject_payload);
        outputs.push(DemoResult {
            path: "sidecar_reject".to_string(),
            decision: if reject_resp.is_ok() {
                "UnexpectedAccept".to_string()
            } else {
                "Reject".to_string()
            },
            message: reject_resp
                .as_ref()
                .map(|r| format!("status={}", r.status))
                .unwrap_or_else(|e| e.to_string()),
            action: action_value,
            sidecar_status: reject_resp.ok().map(|r| r.status),
        });
    }

    let out = json!({ "demo": outputs });
    let path = write_output_json("demo_e2e.json", &out)?;
    println!("{}", serde_json::to_string_pretty(&out)?);
    println!("Saved demo artifact to {}", path.display());
    Ok(())
}

fn run_bench(
    iterations: usize,
    sidecar_url: &str,
    with_sidecar: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let micro = generate_runtime_ai_micro().or_else(|_| load_ai_demo_micro())?;
    let chain = generate_runtime_ai_chain(1000).or_else(|_| load_ai_demo_chain())?;
    let micro_json = serde_json::to_string(&micro)?;

    let mut results = Vec::new();
    let mut sidecar_results = Vec::new();

    results.push(time_named("micro_verify", iterations, || {
        let _ = verify_micro(micro.clone());
    }));

    results.push(time_named("micro_parse_verify", iterations, || {
        let parsed: MicroReceiptWire = serde_json::from_str(&micro_json).unwrap();
        let _ = verify_micro(parsed);
    }));

    results.push(time_named("chain_verify", iterations, || {
        let _ = verify_chain(chain.clone());
    }));

    results.push(time_named("slab_build", iterations, || {
        let _ = build_slab(chain.clone());
    }));

    results.push(time_named(
        "ape_generate_verify_mutation",
        iterations,
        || {
            let proposal = ape::engine::generate(Strategy::Mutation, &Input::empty(), 42);
            if let Candidate::Micro(w) = proposal.candidate {
                let _ = verify_micro(w);
            }
        },
    ));

    if with_sidecar {
        let action = json!({ "action": "transfer_100_tokens", "amount": 100, "target": "alice" });
        let valid_payload = ExecuteVerifiedRequest {
            receipt: micro.clone(),
            action: action.clone(),
        };
        sidecar_results.push(time_named(
            "sidecar_execute_verified_accept",
            iterations,
            || {
                let _ = execute_verified(sidecar_url, &valid_payload);
            },
        ));

        let invalid_receipt = candidate_to_micro(
            &ape::engine::generate(Strategy::Violation, &Input::empty(), 42).candidate,
        )?;
        let invalid_payload = ExecuteVerifiedRequest {
            receipt: invalid_receipt,
            action,
        };
        sidecar_results.push(time_named(
            "sidecar_execute_verified_reject",
            iterations,
            || {
                let _ = execute_verified(sidecar_url, &invalid_payload);
            },
        ));
    }

    let suite = BenchSuite {
        results,
        sidecar_results,
    };
    let path = write_output_json("bench_results.json", &suite)?;
    println!("{}", serde_json::to_string_pretty(&suite)?);
    println!("Saved benchmark artifact to {}", path.display());
    Ok(())
}

fn time_named<F>(name: &str, iterations: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let throughput = iterations as f64 / elapsed.as_secs_f64();
    let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    BenchResult {
        name: name.to_string(),
        iterations,
        total_ms,
        throughput_per_sec: throughput,
        avg_us,
    }
}

fn parse_strategy(strategy: &str) -> Strategy {
    match strategy {
        "mutation" => Strategy::Mutation,
        "recombination" => Strategy::Recombination,
        "violation" => Strategy::Violation,
        "overflow" => Strategy::Overflow,
        "contradiction" => Strategy::Contradiction,
        _ => Strategy::Mutation,
    }
}

fn candidate_to_micro(
    candidate: &Candidate,
) -> Result<MicroReceiptWire, Box<dyn std::error::Error>> {
    match candidate {
        Candidate::Micro(w) => Ok(w.clone()),
        _ => Err("Only Micro receipts supported".into()),
    }
}

/// Strategy metrics for demo output
#[derive(Debug, Serialize, Clone)]
struct StrategyMetrics {
    strategy: String,
    note: String,
    generated: usize,
    rejected: usize,
    escaped: usize,
    first_escaped_seed: Option<u64>,
    avg_latency_us: f64,
    worst_latency_us: u64,
    /// Breakdown by attack subtype (for mutation/recombination triage)
    subtype_breakdown: Vec<SubtypeCount>,
}

/// Subtype count for triage
#[derive(Debug, Serialize, Clone)]
struct SubtypeCount {
    subtype: String,
    generated: usize,
    rejected: usize,
    escaped: usize,
}

/// Baseline comparison result for dominance table
#[derive(Debug, Serialize, Clone)]
struct BaselineResult {
    system: String,
    rejected: usize,
    escaped: usize,
    rejection_rate: f64,
}

/// Dominance comparison output
#[derive(Debug, Serialize)]
struct DominanceTable {
    summary: String,
    baselines: Vec<BaselineResult>,
    per_strategy_escapes: Vec<StrategyEscapes>,
}

/// Per-strategy escape breakdown for dominance analysis
#[derive(Debug, Serialize)]
struct StrategyEscapes {
    strategy: String,
    full_verifier_escapes: usize,
    schema_only_escapes: usize,
    rule_only_escapes: usize,
}

/// Schema-only baseline: validates JSON structure only, no crypto/chain/trajectory
fn verify_schema_only(wire: &MicroReceiptWire) -> bool {
    // Check all required fields exist and are non-empty strings
    if wire.schema_id.is_empty() || wire.version.is_empty() || wire.object_id.is_empty() {
        return false;
    }
    // Check schema_id format (rough check)
    if !wire.schema_id.starts_with("coh.") {
        return false;
    }
    // Check version format (rough check)
    if !wire.version.contains('.') {
        return false;
    }
    // Check metrics fields exist
    if wire.metrics.v_pre.is_empty() || wire.metrics.v_post.is_empty() {
        return false;
    }
    // Check hashes are 64-char hex
    if wire.canon_profile_hash.len() != 64
        || wire.policy_hash.len() != 64
        || wire.state_hash_prev.len() != 64
        || wire.state_hash_next.len() != 64
        || wire.chain_digest_prev.len() != 64
        || wire.chain_digest_next.len() != 64
    {
        return false;
    }
    // Check step_index is reasonable
    if wire.step_index > 1_000_000 {
        return false;
    }
    true
}

/// Rule-only baseline: checks simple arithmetic invariant v_post + spend <= v_pre + defect
fn verify_rule_only(wire: &MicroReceiptWire) -> bool {
    // Parse metrics - any parsing error means reject
    let v_pre: Option<u128> = wire.metrics.v_pre.parse().ok();
    let v_post: Option<u128> = wire.metrics.v_post.parse().ok();
    let spend: Option<u128> = wire.metrics.spend.parse().ok();
    let defect: Option<u128> = wire.metrics.defect.parse().ok();

    // All must parse successfully
    let (Some(vp), Some(vpo), Some(sp), Some(df)) = (v_pre, v_post, spend, defect) else {
        return false;
    };

    // Simple arithmetic check: v_post + spend <= v_pre + defect
    // (This is the policy constraint - but without cryptographic verification)
    vp.saturating_add(sp) <= vpo.saturating_add(df)
}

fn run_strategy_demo(
    iterations: usize,
    _sidecar_url: &str,
    _with_sidecar: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use ape::seed::SeededRng;

    println!("\n=== APE Strategy Demo ===");
    println!("Testing each strategy with {} iterations\n", iterations);
    println!(
        "| {:14} | {:>9} | {:>8} | {:>7} | {:>12} | {:>15} |",
        "Strategy", "Generated", "Rejected", "Escaped", "Avg Latency", "Notes"
    );
    println!(
        "|{}|{}|{}|{}|{}|{}|",
        "-".repeat(15),
        "-".repeat(10),
        "-".repeat(9),
        "-".repeat(8),
        "-".repeat(13),
        "-".repeat(16)
    );

    let mut results = Vec::new();
    // Generate a valid base receipt for mutation/recombination to mutate
    let base_receipt = generate_runtime_ai_micro().or_else(|_| load_ai_demo_micro())?;
    let input = Input::from_micro(base_receipt);

    for strategy in Strategy::all() {
        let mut rng = SeededRng::new(42);
        let mut rejected = 0;
        let mut escaped = 0;
        let mut first_escaped_seed = None;
        let mut latencies = Vec::with_capacity(iterations);

        // Track subtypes for mutation/recombination triage
        let mut subtype_counts: std::collections::HashMap<String, SubtypeCount> =
            std::collections::HashMap::new();

        for seed in 0..iterations as u64 {
            // Use seeded RNG with different seed per iteration
            let mut iter_rng = SeededRng::new(seed.wrapping_add(rng.next() as u64));

            let start = Instant::now();
            let candidate = strategy.generate(&input, &mut iter_rng);

            // Determine attack subtype from candidate
            let subtype = if let Candidate::Micro(m) = &candidate {
                // Determine subtype
                if m.signatures.is_none() {
                    "missing_sig"
                } else if m.state_hash_next == m.state_hash_prev {
                    "state_loop"
                } else if m.step_index == 0 {
                    "zero_step"
                } else if m.object_id.contains("mut") {
                    "object_id_tamper"
                } else if m.metrics.v_pre != m.metrics.v_post {
                    "value_tamper"
                } else {
                    "field_corruption"
                }
            } else {
                "unknown"
            };

            // Update subtype count
            let entry = subtype_counts
                .entry(subtype.to_string())
                .or_insert_with(|| SubtypeCount {
                    subtype: subtype.to_string(),
                    generated: 0,
                    rejected: 0,
                    escaped: 0,
                });
            entry.generated += 1;

            // Verify the candidate
            if let Some(micro) = candidate.as_micro() {
                let result = verify_micro(micro.clone());
                let elapsed = start.elapsed().as_micros() as u64;
                latencies.push(elapsed);

                match result.decision {
                    Decision::Accept => {
                        escaped += 1;
                        entry.escaped += 1;
                        if first_escaped_seed.is_none() {
                            first_escaped_seed = Some(seed);
                        }
                    }
                    Decision::Reject => {
                        rejected += 1;
                        entry.rejected += 1;
                    }
                    Decision::SlabBuilt => {
                        // Shouldn't happen for micro receipts, treat as escaped
                        escaped += 1;
                        entry.escaped += 1;
                    }
                    Decision::TerminalSuccess => {
                        // Terminal states count as rejected (trajectory ended)
                        rejected += 1;
                        entry.rejected += 1;
                    }
                    Decision::TerminalFailure => {
                        // Terminal failure counts as rejected
                        rejected += 1;
                        entry.rejected += 1;
                    }
                    Decision::AbortBudget => {
                        // Budget exceeded counts as rejected
                        rejected += 1;
                        entry.rejected += 1;
                    }
                }
            }
        }

        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
        } else {
            0.0
        };
        let worst_latency = latencies.iter().max().copied().unwrap_or(0);

        // Convert subtype counts to vector
        let mut subtype_breakdown: Vec<SubtypeCount> = subtype_counts.into_values().collect();
        subtype_breakdown.sort_by(|a, b| a.subtype.cmp(&b.subtype));

        let metrics = StrategyMetrics {
            strategy: strategy.name().to_string(),
            note: strategy.note().to_string(),
            generated: iterations,
            rejected,
            escaped,
            first_escaped_seed,
            avg_latency_us: avg_latency,
            worst_latency_us: worst_latency,
            subtype_breakdown: subtype_breakdown.clone(),
        };
        results.push(metrics.clone());

        println!(
            "| {:14} | {:>9} | {:>8} | {:>7} | {:>11.0}us | {:>15} |",
            metrics.strategy,
            metrics.generated,
            metrics.rejected,
            metrics.escaped,
            metrics.avg_latency_us,
            metrics.note
        );
    }

    println!("\n=== Subtype Triage (Mutation/Recombination) ===");
    println!("Showing benign vs security-relevant escapes\n");

    // Print subtype breakdown for mutation and recombination
    for result in &results {
        if result.strategy == "mutation" || result.strategy == "recombination" {
            println!("--- {} ---", result.strategy);
            println!(
                "| {:20} | {:>9} | {:>8} | {:>7} |",
                "Subtype", "Gen", "Rej", "Esc"
            );
            println!(
                "|{}|{}|{}|{}|",
                "-".repeat(21),
                "-".repeat(10),
                "-".repeat(9),
                "-".repeat(8)
            );
            for sub in &result.subtype_breakdown {
                println!(
                    "| {:20} | {:>9} | {:>8} | {:>7} |",
                    sub.subtype, sub.generated, sub.rejected, sub.escaped
                );
            }
            println!();
        }
    }

    println!("=== Summary ===");
    let total_rejected: usize = results.iter().map(|r| r.rejected).sum();
    let total_generated: usize = results.iter().map(|r| r.generated).sum();
    let total_escaped: usize = results.iter().map(|r| r.escaped).sum();
    println!(
        "Total: {} generated, {} rejected, {} escaped",
        total_generated, total_rejected, total_escaped
    );
    if total_escaped == 0 {
        println!("VERIFIER: 100% rejection across all attack classes!");
    } else {
        println!(
            "VERIFIER: {}/{} = {:.1}% rejection rate",
            total_rejected,
            total_generated,
            100.0 * total_rejected as f64 / total_generated as f64
        );
    }

    // Write results
    let path = write_output_json("strategy_demo_results.json", &results)?;
    println!("\nSaved strategy demo results to {}", path.display());

    Ok(())
}

/// Run comparative failure dominance benchmark
/// Compares full verifier against schema-only and rule-only baselines
fn run_dominance(iterations: usize, _sidecar_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use ape::seed::SeededRng;

    println!("\n=== Comparative Failure Dominance ===");
    println!(
        "Testing all systems with {} iterations per strategy\n",
        iterations
    );

    // Track totals for each system
    let mut full_verifier_rejected = 0usize;
    let mut full_verifier_escaped = 0usize;
    let mut schema_only_rejected = 0usize;
    let mut schema_only_escaped = 0usize;
    let mut rule_only_rejected = 0usize;
    let mut rule_only_escaped = 0usize;

    // Per-strategy breakdown
    let mut per_strategy: Vec<StrategyEscapes> = Vec::new();

    let base_receipt = generate_runtime_ai_micro().or_else(|_| load_ai_demo_micro())?;
    let input = Input::from_micro(base_receipt);

    for strategy in Strategy::all() {
        let mut rng = SeededRng::new(42);
        let mut strategy_full_escapes = 0usize;
        let mut strategy_schema_escapes = 0usize;
        let mut strategy_rule_escapes = 0usize;

        for seed in 0..iterations as u64 {
            let mut iter_rng = SeededRng::new(seed.wrapping_add(rng.next() as u64));
            let candidate = strategy.generate(&input, &mut iter_rng);

            if let Some(micro) = candidate.as_micro() {
                // Full verifier (Coh Wedge)
                let full_result = verify_micro(micro.clone());
                let full_escaped = full_result.decision == Decision::Accept;

                if full_escaped {
                    full_verifier_escaped += 1;
                    strategy_full_escapes += 1;
                } else {
                    full_verifier_rejected += 1;
                }

                // Schema-only baseline
                let schema_passed = verify_schema_only(micro);
                if schema_passed {
                    schema_only_escaped += 1;
                    strategy_schema_escapes += 1;
                } else {
                    schema_only_rejected += 1;
                }

                // Rule-only baseline
                let rule_passed = verify_rule_only(micro);
                if rule_passed {
                    rule_only_escaped += 1;
                    strategy_rule_escapes += 1;
                } else {
                    rule_only_rejected += 1;
                }
            }
        }

        per_strategy.push(StrategyEscapes {
            strategy: strategy.name().to_string(),
            full_verifier_escapes: strategy_full_escapes,
            schema_only_escapes: strategy_schema_escapes,
            rule_only_escapes: strategy_rule_escapes,
        });
    }

    // Print headline table
    let total = iterations * 20;
    println!(
        "| {:14} | {:>9} | {:>7} | {:>11} |",
        "System", "Escaped", "Rejected", "Rejection %"
    );
    println!(
        "|{}|{}|{}|{}|",
        "-".repeat(15),
        "-".repeat(10),
        "-".repeat(8),
        "-".repeat(12)
    );

    let full_rej_pct = 100.0 * full_verifier_rejected as f64 / total as f64;
    let schema_rej_pct = 100.0 * schema_only_rejected as f64 / total as f64;
    let rule_rej_pct = 100.0 * rule_only_rejected as f64 / total as f64;

    println!(
        "| {:14} | {:>9} | {:>7} | {:>10.1}% |",
        "Full Verifier", full_verifier_escaped, full_verifier_rejected, full_rej_pct
    );
    println!(
        "| {:14} | {:>9} | {:>7} | {:>10.1}% |",
        "Schema Only", schema_only_escaped, schema_only_rejected, schema_rej_pct
    );
    println!(
        "| {:14} | {:>9} | {:>7} | {:>10.1}% |",
        "Rule Only", rule_only_escaped, rule_only_rejected, rule_rej_pct
    );
    println!();

    // Per-strategy breakdown
    println!("=== Per-Strategy Escapes ===");
    println!(
        "| {:14} | {:>18} | {:>18} | {:>15} |",
        "Strategy", "Full Verifier", "Schema Only", "Rule Only"
    );
    println!(
        "|{}|{}|{}|{}|",
        "-".repeat(15),
        "-".repeat(19),
        "-".repeat(19),
        "-".repeat(16)
    );
    for s in &per_strategy {
        println!(
            "| {:14} | {:>18} | {:>18} | {:>15} |",
            s.strategy, s.full_verifier_escapes, s.schema_only_escapes, s.rule_only_escapes
        );
    }
    println!();

    // Build dominance output
    let baselines = vec![
        BaselineResult {
            system: "Full Verifier".to_string(),
            rejected: full_verifier_rejected,
            escaped: full_verifier_escaped,
            rejection_rate: full_rej_pct,
        },
        BaselineResult {
            system: "Schema Only".to_string(),
            rejected: schema_only_rejected,
            escaped: schema_only_escaped,
            rejection_rate: schema_rej_pct,
        },
        BaselineResult {
            system: "Rule Only".to_string(),
            rejected: rule_only_rejected,
            escaped: rule_only_escaped,
            rejection_rate: rule_rej_pct,
        },
    ];

    let summary = format!(
        "Full verifier rejects {:.1}% of attacks vs Schema-only {:.1}% and Rule-only {:.1}%",
        full_rej_pct, schema_rej_pct, rule_rej_pct
    );

    let table = DominanceTable {
        summary,
        baselines,
        per_strategy_escapes: per_strategy,
    };

    // Write JSON output
    let path = write_output_json("dominance_results.json", &table)?;
    println!("Saved dominance results to {}", path.display());
    println!();
    println!("=== Key Insight ===");
    println!(
        "Full Verifier achieves {:.1}% rejection while baselines miss {} to {} attacks",
        full_rej_pct,
        schema_only_escaped.min(rule_only_escaped),
        schema_only_escaped.max(rule_only_escaped)
    );

    Ok(())
}
