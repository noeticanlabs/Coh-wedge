//! Fixture loader module for APE
//!
//! Provides deterministic loading of test fixtures for adversarial testing.

use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during fixture loading
#[derive(Error, Debug)]
pub enum FixtureError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Fixture not found: {0}")]
    NotFound(String),
}

/// Load a single micro receipt fixture from the fixtures directory
///
/// # Arguments
/// * `name` - Base name of the fixture file (without .json extension)
///
/// # Returns
/// The parsed MicroReceiptWire
pub fn load_micro(name: &str) -> Result<coh_core::MicroReceiptWire, FixtureError> {
    let path = fixtures_path(&format!("{}.json", name));
    let content = fs::read_to_string(&path)?;
    let receipt: coh_core::MicroReceiptWire = serde_json::from_str(&content)?;
    Ok(receipt)
}

/// Load a chain of micro receipts from a .jsonl file
///
/// # Arguments
/// * `name` - Base name of the fixture file (without .jsonl extension)
///
/// # Returns
/// Vec of MicroReceiptWire (one per line)
pub fn load_chain(name: &str) -> Result<Vec<coh_core::MicroReceiptWire>, FixtureError> {
    let path = fixtures_path(&format!("{}.jsonl", name));
    let content = fs::read_to_string(&path)?;

    let mut receipts = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let receipt: coh_core::MicroReceiptWire = serde_json::from_str(line)?;
        receipts.push(receipt);
    }

    Ok(receipts)
}

/// Load a slab receipt fixture
///
/// # Arguments
/// * `name` - Base name of the fixture file (without .json extension)
///
/// # Returns
/// The parsed SlabReceiptWire
pub fn load_slab(name: &str) -> Result<coh_core::SlabReceiptWire, FixtureError> {
    let path = fixtures_path(&format!("{}.json", name));
    let content = fs::read_to_string(&path)?;
    let slab: coh_core::SlabReceiptWire = serde_json::from_str(&content)?;
    Ok(slab)
}

/// Get the canonical fixtures directory path
fn fixtures_path(filename: &str) -> PathBuf {
    // Fixtures are stored relative to the crate root
    let crate_root = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(crate_root).join("fixtures").join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_micro() {
        let receipt = load_micro("valid_micro").expect("Failed to load valid_micro");
        assert_eq!(receipt.schema_id, "coh.receipt.micro.v1");
    }

    #[test]
    fn test_load_valid_chain() {
        let chain = load_chain("valid_chain").expect("Failed to load valid_chain");
        assert!(!chain.is_empty());
    }
}
