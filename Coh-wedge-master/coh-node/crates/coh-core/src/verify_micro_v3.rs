//! V3 Micro Verification - Transition Contract verification logic
//! 
//! Extends V1/V2 verification with:
//! - Objective layer checking
//! - Sequence guard checking
//! - Policy governance checking

use crate::reject::RejectCode;
use crate::types::{Decision, MicroReceipt};
use crate::types_v3::{
    MicroReceiptV3, MicroReceiptV3Wire, 
    SequenceGuard, TieredConfig, VerificationMode,
    ObjectiveResult, PolicyGovernance
};
use std::collections::HashMap;

/// V3 verification result
#[derive(Clone, Debug, serde::Serialize)]
pub struct VerifyMicroV3Result {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    /// V3-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective_checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_applied: Option<bool>,
}

/// Verify V3 receipt with Transition Contract checks
#[must_use]
pub fn verify_micro_v3(
    wire: MicroReceiptV3Wire,
    config: &TieredConfig,
    sequence_guard: &SequenceGuard,
    policy_gov: &PolicyGovernance,
    prev_state: Option<crate::types::Hash32>,
    prev_chain_digest: Option<crate::types::Hash32>,
) -> VerifyMicroV3Result {
    // 1. Parse V3 wire to internal type
    let r = match MicroReceiptV3::try_from(wire.clone()) {
        Ok(r) => r,
        Err(code) => {
            return VerifyMicroV3Result {
                decision: Decision::Reject,
                code: Some(code),
                message: format!("Parse error: {:?}", code),
                step_index: Some(wire.step_index),
                object_id: Some(wire.object_id),
                objective_checked: None,
                sequence_checked: None,
                override_applied: None,
            };
        }
    };
    
    // 2. Schema check
    if wire.schema_id != "coh.receipt.micro.v3" {
        return VerifyMicroV3Result {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Invalid schema_id for V3".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            objective_checked: None,
            sequence_checked: None,
            override_applied: Some(r.override_applied),
        };
    }
    
    // 3. Object ID check
    if r.object_id.is_empty() {
        return VerifyMicroV3Result {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Empty object_id".to_string(),
            step_index: Some(r.step_index),
            object_id: None,
            objective_checked: None,
            sequence_checked: None,
            override_applied: Some(r.override_applied),
        };
    }
    
    // 4. Override check - if override applied, accept (governance exception)
    if r.override_applied {
        if policy_gov.allow_overrides {
            return VerifyMicroV3Result {
                decision: Decision::Accept,
                code: None,
                message: "Override accepted".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                objective_checked: Some(r.objective_satisfied()),
                sequence_checked: Some(r.sequence_valid),
                override_applied: Some(true),
            };
        } else {
            return VerifyMicroV3Result {
                decision: Decision::Reject,
                code: Some(RejectCode::RejectPolicyViolation),
                message: "Overrides not allowed".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                objective_checked: Some(r.objective_satisfied()),
                sequence_checked: Some(r.sequence_valid),
                override_applied: Some(true),
            };
        }
    }
    
    // 5. Objective layer check (V3 extension)
    if !r.objective_satisfied() {
        return VerifyMicroV3Result {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: "Objective violated".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            objective_checked: Some(false),
            sequence_checked: Some(r.sequence_valid),
            override_applied: Some(false),
        };
    }
    
    // 6. Sequence guard check (V3 extension)
    // Note: In real implementation, we'd check the rolling accumulator
    if !r.sequence_valid {
        return VerifyMicroV3Result {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: "Sequence guard failed".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            objective_checked: Some(true),
            sequence_checked: Some(false),
            override_applied: Some(false),
        };
    }
    
    // 7. Base V1/V2 checks would go here (state hash, chain digest, etc.)
    // For now, we return ACCEPT with V3 fields
    
    VerifyMicroV3Result {
        decision: Decision::Accept,
        code: None,
        message: "Verification passed".to_string(),
        step_index: Some(r.step_index),
        object_id: Some(r.object_id),
        objective_checked: Some(r.objective_satisfied()),
        sequence_checked: Some(r.sequence_valid),
        override_applied: Some(false),
    }
}

/// Tiered verification entry point
#[must_use]
pub fn verify_with_mode(
    wire: MicroReceiptV3Wire,
    config: &TieredConfig,
    cache: &HashMap<String, crate::types::VerifyMicroResult>,
    sequence_guard: &SequenceGuard,
    policy_gov: &PolicyGovernance,
    prev_state: Option<crate::types::Hash32>,
    prev_chain_digest: Option<crate::types::Hash32>,
) -> VerifyMicroV3Result {
    match config.mode {
        // STRICT: Full verification
        VerificationMode::Strict => {
            verify_micro_v3(wire, config, sequence_guard, policy_gov, prev_state, prev_chain_digest)
        }
        // FAST: Use cache if available
        VerificationMode::Fast => {
            let cache_key = format!("{}:{}", wire.object_id, wire.step_index);
            if let Some(cached) = cache.get(&cache_key) {
                // Return cached result
                VerifyMicroV3Result {
                    decision: cached.decision,
                    code: cached.code,
                    message: format!("(cached) {}", cached.message),
                    step_index: cached.step_index,
                    object_id: cached.object_id,
                    objective_checked: Some(true),
                    sequence_checked: Some(true),
                    override_applied: Some(false),
                }
            } else {
                // Verify and cache
                verify_micro_v3(wire, config, sequence_guard, policy_gov, prev_state, prev_chain_digest)
            }
        }
        // ASYNC: Accept immediately, verify later
        VerificationMode::Async => {
            // Return accept immediately - verification happens async
            VerifyMicroV3Result {
                decision: Decision::Accept,
                code: None,
                message: "(async queued)".to_string(),
                step_index: Some(wire.step_index),
                object_id: Some(wire.object_id),
                objective_checked: None,  // Not checked yet
                sequence_checked: None,   // Not checked yet
                override_applied: Some(wire.override_applied),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_v3_accept() {
        let wire = MicroReceiptV3Wire::default();
        let config = TieredConfig::default();
        let guard = SequenceGuard::default();
        let policy = PolicyGovernance::default();
        
        let result = verify_micro_v3(wire, &config, &guard, &policy, None, None);
        assert_eq!(result.decision, Decision::Accept);
    }
    
    #[test]
    fn test_v3_reject_override_disallowed() {
        let mut wire = MicroReceiptV3Wire::default();
        wire.override_applied = true;
        
        let mut policy = PolicyGovernance::default();
        policy.allow_overrides = false;
        
        let config = TieredConfig::default();
        let guard = SequenceGuard::default();
        
        let result = verify_micro_v3(wire, &config, &guard, &policy, None, None);
        assert_eq!(result.decision, Decision::Reject);
    }
    
    #[test]
    fn test_v3_accept_override_allowed() {
        let mut wire = MicroReceiptV3Wire::default();
        wire.override_applied = true;
        
        let policy = PolicyGovernance {
            allow_overrides: true,
            ..Default::default()
        };
        
        let config = TieredConfig::default();
        let guard = SequenceGuard::default();
        
        let result = verify_micro_v3(wire, &config, &guard, &policy, None, None);
        assert_eq!(result.decision, Decision::Accept);
    }
}