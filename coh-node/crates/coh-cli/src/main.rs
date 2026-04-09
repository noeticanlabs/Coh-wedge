use clap::{Parser, Subcommand, ValueEnum};
use coh_core::types::*;
use coh_core::verify_micro::verify_micro;
use coh_core::verify_chain::verify_chain;
use coh_core::build_slab::build_slab;
use coh_core::verify_slab::verify_slab;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process;

#[derive(Parser)]
#[command(name = "coh-validator")]
#[command(about = "Coh Constraint Verifier Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a single micro-receipt in isolation
    VerifyMicro {
        input: String,
    },
    /// Verify a chain of micro-receipts from a JSONL file
    VerifyChain {
        input: String,
    },
    /// Build a slab-receipt from a chain of micro-receipts
    BuildSlab {
        input: String,
        #[arg(long, short)]
        out: String,
    },
    /// Verify a standalone slab-receipt
    VerifySlab {
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::VerifyMicro { input } => {
            let wire: MicroReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(e.to_string(), 2, cli.format),
            };
            let res = verify_micro(wire);
            output_result(res, cli.format);
        }
        Commands::VerifyChain { input } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(e.to_string(), 2, cli.format),
            };
            let res = verify_chain(receipts);
            output_result(res, cli.format);
        }
        Commands::BuildSlab { input, out } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(e.to_string(), 2, cli.format),
            };
            let res = build_slab(receipts);
            if res.decision == Decision::Accept {
                if let Some(ref slab) = res.slab {
                    if let Err(e) = save_json(&out, &slab) {
                        exit_with_error(e.to_string(), 3, cli.format);
                    }
                }
                output_result(res, cli.format);
            } else {
                // Exit code 4 if it was a chain failure during build
                let exit_code = if res.code.is_some() { 4 } else { 1 };
                output_result_with_exit(res, cli.format, exit_code);
            }
        }
        Commands::VerifySlab { input } => {
            let wire: SlabReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(e.to_string(), 2, cli.format),
            };
            let res = verify_slab(wire);
            output_result(res, cli.format);
        }
    }
}

fn load_json<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let val = serde_json::from_reader(reader)?;
    Ok(val)
}

fn load_jsonl<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<Vec<T>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let val = serde_json::from_str(&line)?;
        results.push(val);
    }
    Ok(results)
}

fn save_json<T: serde::Serialize>(path: &str, val: &T) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let buf = serde_json::to_vec_pretty(val)?;
    file.write_all(&buf)?;
    Ok(())
}

fn output_result<T: serde::Serialize + IntoDecision>(res: T, format: Format) {
    let exit_code = if res.is_accept() { 0 } else { 1 };
    output_result_with_exit(res, format, exit_code);
}

fn output_result_with_exit<T: serde::Serialize>(res: T, format: Format, exit_code: i32) {
    match format {
        Format::Json => {
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Format::Text => {
            // Basic text output for demo:
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
    }
    process::exit(exit_code);
}

fn exit_with_error(err: String, code: i32, format: Format) -> ! {
    match format {
        Format::Json => {
            let msg = serde_json::json!({ "error": err, "code": code });
            println!("{}", serde_json::to_string_pretty(&msg).unwrap());
        }
        Format::Text => {
            eprintln!("Error: {}", err);
        }
    }
    process::exit(code);
}

trait IntoDecision {
    fn is_accept(&self) -> bool;
}

impl IntoDecision for VerifyMicroResult {
    fn is_accept(&self) -> bool { self.decision == Decision::Accept }
}
impl IntoDecision for VerifyChainResult {
    fn is_accept(&self) -> bool { self.decision == Decision::Accept }
}
impl IntoDecision for BuildSlabResult {
    fn is_accept(&self) -> bool { self.decision == Decision::Accept }
}
impl IntoDecision for VerifySlabResult {
    fn is_accept(&self) -> bool { self.decision == Decision::Accept }
}
