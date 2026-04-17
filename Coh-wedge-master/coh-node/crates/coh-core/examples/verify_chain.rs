//! # Example: Verify a chain from JSONL
//!
//! This example demonstrates loading a chain of receipts from JSONL
//! and verifying the entire sequence.

use coh_core::types::MicroReceiptWire;
use coh_core::{verify_chain, Decision};
use serde_json::from_str;
use std::io::BufRead;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load chain from JSONL file
    let file = std::fs::File::open("examples/chain_valid.jsonl")?;
    let reader = std::io::BufReader::new(file);

    let mut receipts = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            let receipt: MicroReceiptWire = from_str(&line)?;
            receipts.push(receipt);
        }
    }

    println!("Loaded {} receipts from chain", receipts.len());

    // Verify the chain
    let result = verify_chain(receipts);

    match result.decision {
        Decision::Accept => {
            println!("✓ Chain verified successfully");
            println!("  Steps verified: {}", result.steps_verified);
            println!("  First step: {}", result.first_step_index);
            println!("  Last step: {}", result.last_step_index);
            println!(
                "  Final digest: {}",
                result.final_chain_digest.unwrap_or_default()
            );
        }
        Decision::Reject => {
            println!("✗ Chain rejected");
            println!("  Code: {:?}", result.code);
            println!("  Message: {}", result.message);
            println!("  Failed at step: {:?}", result.failing_step_index);
            println!(
                "  Verified before failure: {:?}",
                result.steps_verified_before_failure
            );
        }
        _ => unreachable!(),
    }

    Ok(())
}
