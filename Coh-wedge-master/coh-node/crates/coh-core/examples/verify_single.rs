//! # Example: Verify a single receipt from JSON
//!
//! This example demonstrates loading a micro-receipt from JSON
//! and verifying it using the coh-core API.

use coh_core::types::MicroReceiptWire;
use coh_core::{verify_micro, Decision};
use serde_json::from_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load receipt from JSON file
    let json = std::fs::read_to_string("examples/micro_valid.json")?;
    let receipt: MicroReceiptWire = from_str(&json)?;

    // Verify the receipt
    let result = verify_micro(receipt);

    match result.decision {
        Decision::Accept => {
            println!("✓ Receipt verified successfully");
            println!("  Step index: {:?}", result.step_index);
            println!("  Object ID: {:?}", result.object_id);
            println!("  Next digest: {:?}", result.chain_digest_next);
        }
        Decision::Reject => {
            println!("✗ Receipt rejected");
            println!("  Code: {:?}", result.code);
            println!("  Message: {}", result.message);
        }
        _ => unreachable!(),
    }

    Ok(())
}
