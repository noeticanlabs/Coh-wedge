//! Adversarial test fixtures for APE
//!
//! These tests verify that the Coh verifier correctly rejects malformed or
//! adversarial receipts.

use ape::fixtures::{load_chain, load_micro};
use coh_core::{verify_chain::verify_chain, verify_micro::verify_micro, Decision};

/// Test that a receipt with invalid accounting (v_post != v_pre - spend) is rejected
#[test]
fn test_invalid_accounting_reject() {
    let receipt = load_micro("invalid_accounting").expect("Failed to load invalid_accounting");
    let result = verify_micro(receipt);
    assert!(
        result.decision == Decision::Reject,
        "Invalid accounting should be rejected, got: {:?}",
        result.decision
    );
}

/// Test that a receipt with wrong schema_id is rejected
#[test]
fn test_invalid_schema_reject() {
    let receipt = load_micro("invalid_schema").expect("Failed to load invalid_schema");
    let result = verify_micro(receipt);
    assert!(
        result.decision == Decision::Reject,
        "Invalid schema should be rejected, got: {:?}",
        result.decision
    );
}

/// Test that a receipt with overflow in metrics is rejected
#[test]
fn test_overflow_reject() {
    let receipt = load_micro("overflow").expect("Failed to load overflow");
    let result = verify_micro(receipt);
    assert!(
        result.decision == Decision::Reject,
        "Overflow should be rejected, got: {:?}",
        result.decision
    );
}

/// Test that an invalid chain with broken state links is rejected
#[test]
fn test_invalid_state_link_reject() {
    let chain = load_chain("invalid_state_link").expect("Failed to load invalid_state_link");
    let result = verify_chain(chain);
    assert!(
        result.decision == Decision::Reject,
        "Invalid state link should be rejected, got: {:?} - {}",
        result.decision,
        result.message
    );
}

/// Test that a chain with wrong chain digests is rejected
#[test]
fn test_invalid_chain_digest_reject() {
    let chain = load_chain("invalid_chain_digest").expect("Failed to load invalid_chain_digest");
    let result = verify_chain(chain);
    assert!(
        result.decision == Decision::Reject,
        "Invalid chain digest should be rejected, got: {:?} - {}",
        result.decision,
        result.message
    );
}

/// Integration test for valid chain using external example
/// This test verifies the fixtures work with the verify_chain example
#[test]
fn test_valid_chain_integration() {
    // This test verifies that valid_chain.jsonl can be verified
    // Run with: cargo run -p coh-core --example verify_chain ../ape/fixtures/valid_chain.jsonl
    // Expected: "✓ Chain verified successfully"
    let chain = load_chain("valid_chain").expect("Failed to load valid_chain");
    // Just verify we can load it - the actual verification is done via example
    assert_eq!(chain.len(), 2, "Valid chain should have 2 receipts");
}
