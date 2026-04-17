use clap::{Parser, Subcommand, ValueEnum};
use coh_core::build_slab::build_slab;
use coh_core::types::*;
use coh_core::verify_chain::verify_chain;
use coh_core::verify_micro::verify_micro;
use coh_core::verify_slab_envelope;
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
    VerifyMicro { input: String },
    /// Verify a chain of micro-receipts from a JSONL file
    VerifyChain { input: String },
    /// Build a slab-receipt from a chain of micro-receipts
    BuildSlab {
        input: String,
        #[arg(long, short)]
        out: String,
    },
    /// Verify a standalone slab-receipt
    VerifySlab { input: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::VerifyMicro { input } => {
            let wire: MicroReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(
                    format!("Failed to load micro-receipt from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_micro(wire);
            output_result(res, cli.format);
        }
        Commands::VerifyChain { input } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(
                    format!("Failed to load chain from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_chain(receipts);
            output_result(res, cli.format);
        }
        Commands::BuildSlab { input, out } => {
            let receipts = match load_jsonl(&input) {
                Ok(r) => r,
                Err(e) => exit_with_error(
                    format!("Failed to load source chain from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let mut res = build_slab(receipts);
            if res.decision == Decision::SlabBuilt {
                if let Some(ref slab) = res.slab {
                    if let Err(e) = save_json(&out, &slab) {
                        exit_with_error(
                            format!("Failed to save slab to {}: {}", out, e),
                            3,
                            cli.format,
                        );
                    }
                    res.output = Some(out.clone());
                }
                output_result(res, cli.format);
            } else {
                let exit_code = if let Some(code) = &res.code {
                    match code {
                        RejectCode::RejectChainDigest | RejectCode::RejectStateHashLink => 4,
                        RejectCode::RejectSchema if res.message.contains("Index discontinuity") => {
                            4
                        }
                        _ => 1,
                    }
                } else {
                    1
                };
                output_result_with_exit(res, cli.format, exit_code);
            }
        }
        Commands::VerifySlab { input } => {
            let wire: SlabReceiptWire = match load_json(&input) {
                Ok(w) => w,
                Err(e) => exit_with_error(
                    format!("Failed to load slab-receipt from {}: {}", input, e),
                    2,
                    cli.format,
                ),
            };
            let res = verify_slab_envelope(wire);
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
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        // Overbuilt Parser: Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match serde_json::from_str::<T>(trimmed) {
            Ok(val) => results.push(val),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Line {}: JSON parsing failed: {}",
                    i + 1,
                    e
                ))
            }
        }
    }
    if results.is_empty() {
        return Err(anyhow::anyhow!(
            "File is empty or contains no valid records"
        ));
    }
    Ok(results)
}

fn save_json<T: serde::Serialize>(path: &str, val: &T) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let buf = serde_json::to_vec_pretty(val)?;
    file.write_all(&buf)?;
    Ok(())
}

fn output_result<T: serde::Serialize + DisplayResult>(res: T, format: Format) {
    let exit_code = if res.is_accept() { 0 } else { 1 };
    output_result_with_exit(res, format, exit_code);
}

fn output_result_with_exit<T: serde::Serialize + DisplayResult>(
    res: T,
    format: Format,
    exit_code: i32,
) {
    match format {
        Format::Json => {
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Format::Text => {
            print!("{}", res.to_text());
        }
    }
    process::exit(exit_code);
}

fn exit_with_error(err: String, code: i32, format: Format) -> ! {
    match format {
        Format::Json => {
            let msg = serde_json::json!({
                "decision": "REJECT",
                "code": "RejectNumericParse",
                "message": err
            });
            println!("{}", serde_json::to_string_pretty(&msg).unwrap());
        }
        Format::Text => {
            println!("REJECT");
            println!("code: RejectNumericParse");
            println!("message: {}", err);
        }
    }
    process::exit(code);
}

trait DisplayResult {
    fn is_accept(&self) -> bool;
    fn to_text(&self) -> String;
}

fn decision_to_text(d: &Decision) -> String {
    match d {
        Decision::Accept => "ACCEPT".to_string(),
        Decision::Reject => "REJECT".to_string(),
        Decision::SlabBuilt => "SLAB_BUILT".to_string(),
        Decision::TerminalSuccess => "TERMINAL_SUCCESS".to_string(),
        Decision::TerminalFailure => "TERMINAL_FAILURE".to_string(),
        Decision::AbortBudget => "ABORT_BUDGET".to_string(),
    }
}

impl DisplayResult for VerifyMicroResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        if let Some(idx) = self.step_index {
            s.push_str(&format!("step_index: {}\n", idx));
        }
        if let Some(oid) = &self.object_id {
            s.push_str(&format!("object_id: {}\n", oid));
        }
        if let Some(digest) = &self.chain_digest_next {
            s.push_str(&format!("chain_digest_next: {}\n", digest));
        }
        s
    }
}

impl DisplayResult for VerifyChainResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        s.push_str(&format!("steps_verified: {}\n", self.steps_verified));
        s.push_str(&format!("first_step_index: {}\n", self.first_step_index));
        s.push_str(&format!("last_step_index: {}\n", self.last_step_index));
        if let Some(digest) = &self.final_chain_digest {
            s.push_str(&format!("final_chain_digest: {}\n", digest));
        }
        if let Some(fidx) = self.failing_step_index {
            s.push_str(&format!("failing_step_index: {}\n", fidx));
        }
        s
    }
}

impl DisplayResult for BuildSlabResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::SlabBuilt
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        } else {
            s.push_str(&format!("message: {}\n", self.message));
        }
        if let Some(rs) = self.range_start {
            s.push_str(&format!("range_start: {}\n", rs));
        }
        if let Some(re) = self.range_end {
            s.push_str(&format!("range_end: {}\n", re));
        }
        if let Some(mc) = self.micro_count {
            s.push_str(&format!("micro_count: {}\n", mc));
        }
        if let Some(root) = &self.merkle_root {
            s.push_str(&format!("merkle_root: {}\n", root));
        }
        if let Some(out) = &self.output {
            s.push_str(&format!("output: {}\n", out));
        }
        s
    }
}

impl DisplayResult for VerifySlabResult {
    fn is_accept(&self) -> bool {
        self.decision == Decision::Accept
    }
    fn to_text(&self) -> String {
        let mut s = format!("{}\n", decision_to_text(&self.decision));
        if self.decision == Decision::Reject {
            if let Some(code) = &self.code {
                s.push_str(&format!("code: {:?}\n", code));
            }
            s.push_str(&format!("message: {}\n", self.message));
        }
        s.push_str(&format!("range_start: {}\n", self.range_start));
        s.push_str(&format!("range_end: {}\n", self.range_end));
        if let Some(mc) = self.micro_count {
            s.push_str(&format!("micro_count: {}\n", mc));
        }
        if let Some(root) = &self.merkle_root {
            s.push_str(&format!("merkle_root: {}\n", root));
        }
        s
    }
}
