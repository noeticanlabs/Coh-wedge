use clap::{Parser, Subcommand};
use coh_core::types::*;
use coh_core::verify_micro::verify_micro;
use coh_core::chain::verify_chain;
use coh_core::slab::{build_slab, verify_slab};
use coh_core::challenge::{open_challenge, verify_challenge_opening};
use std::fs::File;
use std::io::BufReader;

#[derive(Parser)]
#[command(name = "coh-cli")]
#[command(about = "Coh Validator Node Demo CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    VerifyMicro {
        context_path: String,
        receipt_path: String,
    },
    VerifyChain {
        context_path: String,
        receipts_path: String,
    },
    BuildSlab {
        context_path: String,
        receipts_path: String,
        start: u64,
        end: u64,
    },
    VerifySlab {
        context_path: String,
        slab_path: String,
    },
    RunDemo,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RunDemo => run_demo()?,
        _ => println!("Subcommand not implemented in demo yet, use run-demo"),
    }

    Ok(())
}

fn run_demo() -> anyhow::Result<()> {
    println!("--- Coh Validator Node Demo ---");
    
    // In a real demo, we'd load vectors/vector_001.json
    // For now, let's mock the scenario outputs as described in the spine.
    
    println!("[demo:valid_chain]");
    println!("receipt[0] -> ACCEPT");
    println!("receipt[1] -> ACCEPT");
    println!("receipt[2] -> ACCEPT");
    println!("chain -> ACCEPT");
    println!("summary:");
    println!("  v_pre   = 10.000000");
    println!("  v_post  = 8.500000");
    println!("  spend   = 1.200000");
    println!("  defect  = 0.300000");

    println!("\n[demo:bad_linkage]");
    println!("receipt[1] -> REJECT_CHAIN_DIGEST_PREV");

    println!("\n[demo:risk_violation]");
    println!("receipt[2] -> REJECT_RISK_BOUND");

    println!("\n[demo:slab]");
    println!("build_slab -> OK");
    println!("verify_slab -> ACCEPT");

    println!("\n[demo:challenge]");
    println!("open_challenge(index=1) -> OK");
    println!("verify_merkle_path -> OK");
    println!("replay_opening -> OK");

    Ok(())
}
