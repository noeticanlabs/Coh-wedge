//! V3 Types - Transition Contract extensions
//!
//! Extends V1/V2 types with:
//! - Objective layer (objective_result, optional)
//! - Sequence guard (sequence_valid)
//! - Policy governance (override_applied)

use crate::reject::RejectCode;
use crate::types::{Hash32, Metrics, MicroReceipt, MicroReceiptWire};
use serde::{Deserialize, Serialize};

/// Objective target types for V3
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveTarget {
    MinimizeSpend,
    MaximizeValue,
    CloseTickets,
    ZeroPending,
    Custom(String),
}

impl Default for ObjectiveTarget {
    fn default() -> Self {
        ObjectiveTarget::MinimizeSpend
    }
}

/// Objective result if checked
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveResult {
    Satisfied(ObjectiveTarget),
    Violated(ObjectiveTarget),
    NotApplicable,
}

impl Default for ObjectiveResult {
    fn default() -> Self {
        ObjectiveResult::NotApplicable
    }
}

/// V3 MicroReceipt - extends V1/V2 with Transition Contract fields
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MicroReceiptV3Wire {
    // Base V2 fields
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<crate::types::SignatureWire>>,
    pub state_hash_prev: String,
    pub state_hash_next: String,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub metrics: crate::types::MetricsWire,
    // V3 Transition Contract fields
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_valid: bool,
    pub override_applied: bool,
}

impl Default for MicroReceiptV3Wire {
    fn default() -> Self {
        Self {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: String::new(),
            canon_profile_hash: String::new(),
            policy_hash: String::new(),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: String::new(),
            state_hash_next: String::new(),
            chain_digest_prev: String::new(),
            chain_digest_next: String::new(),
            metrics: crate::types::MetricsWire::default(),
            objective_result: None,
            sequence_valid: true,
            override_applied: false,
        }
    }
}

/// Internal V3 receipt type
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MicroReceiptV3 {
    // Base fields
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<crate::types::SignatureWire>>,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub metrics: Metrics,
    // V3 Transition Contract fields
    pub objective_result: Option<ObjectiveResult>,
    pub sequence_valid: bool,
    pub override_applied: bool,
}

impl Default for MicroReceiptV3 {
    fn default() -> Self {
        Self {
            schema_id: "coh.receipt.micro.v3".to_string(),
            version: "1.0.0".to_string(),
            object_id: String::new(),
            canon_profile_hash: Hash32::default(),
            policy_hash: Hash32::default(),
            step_index: 0,
            step_type: None,
            signatures: None,
            state_hash_prev: Hash32::default(),
            state_hash_next: Hash32::default(),
            chain_digest_prev: Hash32::default(),
            chain_digest_next: Hash32::default(),
            metrics: Metrics::default(),
            objective_result: None,
            sequence_valid: true,
            override_applied: false,
        }
    }
}

impl TryFrom<MicroReceiptV3Wire> for MicroReceiptV3 {
    type Error = RejectCode;

    fn try_from(w: MicroReceiptV3Wire) -> Result<Self, Self::Error> {
        Ok(MicroReceiptV3 {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            step_index: w.step_index,
            step_type: w.step_type,
            signatures: w.signatures,
            state_hash_prev: Hash32::from_hex(&w.state_hash_prev)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            state_hash_next: Hash32::from_hex(&w.state_hash_next)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)
                .map_err(|_| RejectCode::RejectNumericParse)?,
            metrics: w.metrics.try_into()?,
            // V3 fields
            objective_result: w.objective_result,
            sequence_valid: w.sequence_valid,
            override_applied: w.override_applied,
        })
    }
}

impl MicroReceiptV3 {
    /// Check if objective layer is satisfied (null = not checked = pass)
    pub fn objective_satisfied(&self) -> bool {
        match &self.objective_result {
            None => true, // not checked
            Some(result) => match result {
                ObjectiveResult::Satisfied(_) => true,
                ObjectiveResult::Violated(_) => false,
                ObjectiveResult::NotApplicable => true,
            },
        }
    }

    /// Full V3 validity check
    pub fn is_valid(&self) -> bool {
        // V1/V2 checks...
        !self.object_id.is_empty()
            && self.schema_id == "coh.receipt.micro.v3"
            && self.sequence_valid
            && !self.override_applied
            && self.objective_satisfied()
    }
}

/// Sequence guard configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceGuard {
    pub max_cumulative_spend: u128,
    pub window_size: u64,
    pub max_state_drift: u128,
    pub require_monotonicity: bool,
}

impl Default for SequenceGuard {
    fn default() -> Self {
        Self {
            max_cumulative_spend: u128::MAX,
            window_size: 100,
            max_state_drift: u128::MAX,
            require_monotonicity: false,
        }
    }
}

/// Strict sequence guard
pub fn strict_sequence_guard() -> SequenceGuard {
    SequenceGuard {
        max_cumulative_spend: 10_000,
        window_size: 10,
        max_state_drift: 5_000,
        require_monotonicity: true,
    }
}

/// Policy governance configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGovernance {
    pub policy_version: u64,
    pub policy_chain_valid: bool,
    pub allow_overrides: bool,
}

impl Default for PolicyGovernance {
    fn default() -> Self {
        Self {
            policy_version: 0,
            policy_chain_valid: true,
            allow_overrides: false,
        }
    }
}

/// Verification mode
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMode {
    Strict, // Full verification
    Fast,   // Cached/partial
    Async,  // Post-check
}

impl Default for VerificationMode {
    fn default() -> Self {
        VerificationMode::Strict
    }
}

/// Tiered verification config
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TieredConfig {
    pub mode: VerificationMode,
    pub cache_ttl_seconds: u64,
    pub async_queue_size: u64,
}

impl Default for TieredConfig {
    fn default() -> Self {
        Self {
            mode: VerificationMode::Strict,
            cache_ttl_seconds: 0,
            async_queue_size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3_default() {
        let wire = MicroReceiptV3Wire::default();
        assert_eq!(wire.schema_id, "coh.receipt.micro.v3");
        assert!(wire.sequence_valid);
        assert!(!wire.override_applied);
    }

    #[test]
    fn test_objective_satisfied() {
        let mut receipt = MicroReceiptV3::default();
        assert!(receipt.objective_satisfied());

        receipt.objective_result = Some(ObjectiveResult::Violated(ObjectiveTarget::MinimizeSpend));
        assert!(!receipt.objective_satisfied());
    }

    #[test]
    fn test_sequence_guard_defaults() {
        let guard = SequenceGuard::default();
        assert_eq!(guard.window_size, 100);
    }
}
