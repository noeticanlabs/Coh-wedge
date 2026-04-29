use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("========================================================");
    println!("NPE-Lean-PhaseLoom Benchmark Package Runner v0.1");
    println!("========================================================");

    // Ensure target directory exists
    let target_dir = Path::new("target/npe_wbt/lean_phaseloom");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).expect("Failed to create target directory");
    }

    let examples = vec![
        "phaseloom_lean_loop",
        "phaseloom_pairwise_add_loop",
        "phaseloom_exists_lt_loop",
        "phaseloom_rebuild_pairwise_add_loop",
    ];

    for example in examples {
        println!("\n>>> Running stage: {}...", example);
        let status = Command::new("cargo")
            .args(&["run", "-p", "coh-genesis", "--example", example])
            .status()
            .unwrap_or_else(|_| panic!("Failed to execute {}", example));

        if !status.success() {
            println!("Warning: {} returned non-zero status", example);
        }
    }

    println!("\n========================================================");
    println!("Benchmark execution complete.");
    println!("Artifacts exported to: target/npe_wbt/lean_phaseloom/");
    println!("- summary.md");
    println!("- results.json");
    println!("- results.csv");
    println!("- proof_graph.json");
    println!("- receipts.jsonl");
    println!("- strategy_weights_before.json");
    println!("- strategy_weights_after.json");
    println!("========================================================");
}
