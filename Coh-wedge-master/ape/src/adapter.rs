//! LLM Adapter for APE
//!
//! Provides trait for LLM integration and mock implementation.

use crate::proposal::Candidate;
use crate::seed::SeededRng;
use thiserror::Error;

/// Errors from adapter
#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Network error: {0}")]
    Network(String),
}

/// LLM Response from adapter
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub step_index: u64,
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
    pub state_hash_prev: String,
    pub state_hash_next: String,
}

impl LlmResponse {
    /// Convert to micro receipt
    pub fn to_micro(&self) -> coh_core::types::MicroReceiptWire {
        use coh_core::types::MetricsWire;

        let metrics = MetricsWire {
            v_pre: self.v_pre.to_string(),
            v_post: self.v_post.to_string(),
            spend: self.spend.to_string(),
            defect: self.defect.to_string(),
        };

        let mut wire = coh_core::types::MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: format!("llm.step.{}", self.step_index),
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
                .to_string(),
            policy_hash: "0".repeat(64),
            step_index: self.step_index,
            step_type: Some("llm_step".to_string()),
            signatures: None,
            state_hash_prev: self.state_hash_prev.clone(),
            state_hash_next: self.state_hash_next.clone(),
            chain_digest_prev: "0".repeat(64),
            chain_digest_next: "0".repeat(64),
            metrics,
        };

        // Seal with valid digest if possible
        use coh_core::canon::{to_canonical_json_bytes, to_prehash_view};
        use coh_core::hash::compute_chain_digest;
        use std::convert::TryFrom;

        if let Ok(r) = coh_core::types::MicroReceipt::try_from(wire.clone()) {
            let prehash = to_prehash_view(&r);
            if let Ok(bytes) = to_canonical_json_bytes(&prehash) {
                let digest = compute_chain_digest(r.chain_digest_prev, &bytes);
                wire.chain_digest_next = digest.to_hex();
            }
        }

        wire
    }
}

/// LLM Adapter trait
pub trait LlmAdapter: Send + Sync {
    /// Generate response from prompt
    fn generate(&self, prompt: &str) -> Result<LlmResponse, AdapterError>;
}

/// Mock LLM Adapter for testing (deterministic)
pub struct MockLlmAdapter {
    seed: u64,
    produce_invalid: bool, // If true, simulate hallucination
}

impl MockLlmAdapter {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            produce_invalid: false,
        }
    }

    pub fn with_invalid(seed: u64) -> Self {
        Self {
            seed,
            produce_invalid: true,
        }
    }
}

impl LlmAdapter for MockLlmAdapter {
    fn generate(&self, prompt: &str) -> Result<LlmResponse, AdapterError> {
        let mut rng = SeededRng::new(self.seed);

        // Parse step from prompt or use seed
        let step = if prompt.contains("step") {
            prompt
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        } else {
            self.seed as u64
        };

        // Simulate LLM behavior: consumes resources, outputs state
        let v_pre = 100u128;
        let spend = rng.next() as u128 % 30 + 1;

        // Decide valid or hallucinated
        let (v_post, defect) = if self.produce_invalid && rng.next() % 2 == 0 {
            // Hallucination: claim value higher than should be
            (v_pre + 50, 0) // Invalid!
        } else {
            // Normal: spend creates deficit
            (v_pre.saturating_sub(spend), 0)
        };

        Ok(LlmResponse {
            step_index: step,
            v_pre,
            v_post,
            spend,
            defect,
            state_hash_prev: format!("{:064x}", step),
            state_hash_next: format!("{:064x}", step + 1),
        })
    }
}

/// Adapter wrapper that converts LLM to Candidate
pub fn llm_to_candidate(adapter: &dyn LlmAdapter, prompt: &str) -> Result<Candidate, AdapterError> {
    let response = adapter.generate(prompt)?;
    Ok(Candidate::Micro(response.to_micro()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_valid() {
        let adapter = MockLlmAdapter::new(42);
        let result = adapter.generate("test step 1");

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.step_index, 1);
    }

    #[test]
    fn test_mock_hallucination() {
        let adapter = MockLlmAdapter::with_invalid(42);
        let result = adapter.generate("test step 1");

        // May produce valid or invalid, but shouldn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_determinism() {
        let a1 = MockLlmAdapter::new(42);
        let a2 = MockLlmAdapter::new(42);

        let r1 = a1.generate("step 1").unwrap();
        let r2 = a2.generate("step 1").unwrap();

        assert_eq!(r1.v_pre, r2.v_pre);
        assert_eq!(r1.v_post, r2.v_post);
    }
}
