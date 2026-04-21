//! Execution Layer - Proves execution happened correctly

use crate::reject::RejectCode;
use crate::types::Decision;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

/// Execution mode determines whether state is actually mutated
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    DryRun,
    Real,
    Simulation,
}

/// Action to be executed after verification
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub action_type: String,
    pub target: String,
    pub params: BTreeMap<String, serde_json::Value>,
    pub authority: String,
}

/// Execution proof - proves an action was executed correctly
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionProof {
    pub schema_id: String,
    pub version: String,
    pub parent_receipt_hash: String,
    pub action_result: ActionResultWire,
    pub execution_timestamp: u64,
    pub state_hash_prev: String,
    pub state_hash_next: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionResultWire {
    pub status: String,
    pub state_prev: String,
    pub state_next: String,
}

/// Unified response for execution requests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub decision: Decision,
    pub execution_proof: Option<ExecutionProof>,
    pub state_prev: Option<String>,
    pub state_next: Option<String>,
    pub error: Option<String>,
    pub error_code: Option<RejectCode>,
}

/// State store for tracking state transitions
pub struct StateStore {
    states: HashMap<String, Vec<State>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    pub value: serde_json::Value,
    pub hash: String,
    pub version: u64,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<State> {
        self.states.get(key).and_then(|h| h.last()).cloned()
    }

    pub fn set(&mut self, key: &str, mut state: State) {
        let history = self.states.entry(key.to_string()).or_default();
        state.version = history.len() as u64;
        history.push(state);
    }

    pub fn history(&self, key: &str) -> Vec<State> {
        self.states.get(key).cloned().unwrap_or_default()
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution engine - executes actions after verification
pub struct ExecutionEngine {
    state_store: StateStore,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        Self {
            state_store: StateStore::new(),
        }
    }

    /// Execute an action with verification
    pub fn execute(
        &mut self,
        receipt: crate::types::MicroReceiptWire,
        action: Action,
        mode: ExecutionMode,
    ) -> ExecuteResponse {
        // Step 1: Verify the receipt
        let verify_result = crate::verify_micro(receipt.clone());

        if verify_result.decision != Decision::Accept {
            return ExecuteResponse {
                decision: Decision::Reject,
                execution_proof: None,
                state_prev: None,
                state_next: None,
                error: Some(verify_result.message),
                error_code: verify_result.code,
            };
        }

        // Step 2: Get current state
        let state_key = receipt.object_id.clone();
        let state_prev = self
            .state_store
            .get(&state_key)
            .map(|s| s.hash.clone())
            .unwrap_or_else(|| receipt.state_hash_prev.clone());

        // Step 3: Execute based on mode
        let state_next = match mode {
            ExecutionMode::DryRun => match compute_next_state(&state_prev, &action) {
                Ok(next) => next,
                Err(e) => return ExecuteResponse {
                    decision: Decision::Reject,
                    execution_proof: None,
                    state_prev: Some(state_prev),
                    state_next: None,
                    error: Some(format!("Execution hashing failed: {:?}", e)),
                    error_code: Some(e),
                },
            },
            ExecutionMode::Real => {
                let next = match compute_next_state(&state_prev, &action) {
                    Ok(next) => next,
                    Err(e) => return ExecuteResponse {
                        decision: Decision::Reject,
                        execution_proof: None,
                        state_prev: Some(state_prev),
                        state_next: None,
                        error: Some(format!("Execution hashing failed: {:?}", e)),
                        error_code: Some(e),
                    },
                };
                
                // CRITICAL RIGOR CHECK: Compare computed hash with receipt's claimed next state
                if next != receipt.state_hash_next {
                    return ExecuteResponse {
                        decision: Decision::Reject,
                        execution_proof: None,
                        state_prev: Some(state_prev),
                        state_next: Some(next.clone()),
                        error: Some(format!("Semantic execution mismatch: computed {} but receipt claimed {}", next, receipt.state_hash_next)),
                        error_code: Some(RejectCode::RejectSemanticExecutionMismatch),
                    };
                }

                let new_state = State {
                    id: state_key.clone(),
                    value: serde_json::json!({ "last_action": action.action_type, "params": action.params }),
                    hash: next.clone(),
                    version: 0, // Version will be set by StateStore::set
                };
                self.state_store.set(&state_key, new_state);
                next
            }
            ExecutionMode::Simulation => match compute_next_state(&state_prev, &action) {
                Ok(next) => next,
                Err(e) => return ExecuteResponse {
                    decision: Decision::Reject,
                    execution_proof: None,
                    state_prev: Some(state_prev),
                    state_next: None,
                    error: Some(format!("Execution hashing failed: {:?}", e)),
                    error_code: Some(e),
                },
            },
        };

        // Step 4: Generate execution proof
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proof = ExecutionProof {
            schema_id: "coh.receipt.execution.v1".to_string(),
            version: "1.0.0".to_string(),
            parent_receipt_hash: receipt.chain_digest_next.clone(),
            action_result: ActionResultWire {
                status: "success".to_string(),
                state_prev: state_prev.clone(),
                state_next: state_next.clone(),
            },
            execution_timestamp: timestamp,
            state_hash_prev: state_prev.clone(),
            state_hash_next: state_next.clone(),
        };

        ExecuteResponse {
            decision: Decision::Accept,
            execution_proof: Some(proof),
            state_prev: Some(state_prev),
            state_next: Some(state_next),
            error: None,
            error_code: None,
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute next state hash based on action parameters and previous state.
/// Uses RFC 8785 (JCS) for deterministic canonicalization and SHA-256 for hashing.
fn compute_next_state(current_state_hash: &str, action: &Action) -> Result<String, RejectCode> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(current_state_hash.as_bytes());
    hasher.update(action.action_type.as_bytes());
    hasher.update(action.target.as_bytes());

    // Deterministically serialize params using JCS
    let param_bytes = serde_jcs::to_vec(&action.params).map_err(|_| RejectCode::RejectNumericParse)?;
    hasher.update(&param_bytes);

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_engine_dry_run() {
        let mut engine = ExecutionEngine::new();

        // Use a receipt with signatures that passes verification
        let receipt = crate::types::MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "agent.workflow.demo".to_string(),
            public_key: None,
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
                .to_string(),
            policy_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            step_index: 0,
            step_type: Some("workflow".to_string()),
            signatures: Some(vec![crate::types::SignatureWire {
                signature: "sig-0000000000000000".to_string(),
                signer: "fixture-signer-0".to_string(),
                public_key: None,
                timestamp: 1700000000,
            }]),
            state_hash_prev: "1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            state_hash_next: "2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
            chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            chain_digest_next: "d6439d130820e2e0cf58f082db4b5d39ebed5dba4b30164f3a1955ca635357ba"
                .to_string(),
            metrics: crate::types::MetricsWire {
                v_pre: "100".to_string(),
                v_post: "88".to_string(),
                spend: "12".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        };

        let action = Action {
            action_type: "test_action".to_string(),
            target: "test_target".to_string(),
            params: HashMap::new(),
            authority: "test_authority".to_string(),
        };

        let result = engine.execute(receipt, action, ExecutionMode::DryRun);

        assert_eq!(result.decision, Decision::Accept);
        assert!(result.execution_proof.is_some());

        // Verify proof contents
        let proof = result.execution_proof.unwrap();
        assert_eq!(proof.schema_id, "coh.receipt.execution.v1");
        assert_eq!(
            proof.parent_receipt_hash,
            "d6439d130820e2e0cf58f082db4b5d39ebed5dba4b30164f3a1955ca635357ba"
        );
    }

    #[test]
    fn test_state_store() {
        let mut store = StateStore::new();

        let state = State {
            id: "test".to_string(),
            value: serde_json::json!({ "key": "value" }),
            hash: "abc123".to_string(),
            version: 1,
        };

        store.set("test_key", state.clone());

        assert_eq!(store.get("test_key"), Some(state));
    }

    #[test]
    fn test_execution_rejected_receipt() {
        let mut engine = ExecutionEngine::new();

        // Use an invalid receipt (missing signatures)
        let receipt = crate::types::MicroReceiptWire {
            schema_id: "coh.receipt.micro.v1".to_string(),
            version: "1.0.0".to_string(),
            object_id: "test_obj".to_string(),
            public_key: None,
            canon_profile_hash: "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
                .to_string(),
            policy_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            step_index: 0,
            step_type: None,
            signatures: None, // Missing signatures - will be rejected
            state_hash_prev: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            state_hash_next: "0000000000000000000000000000000000000000000000000000000000000001"
                .to_string(),
            chain_digest_prev: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            chain_digest_next: "03e3fb655ac06d124267f0beb32ee7edc6c770571cf3fb48be83f4d704a50127"
                .to_string(),
            metrics: crate::types::MetricsWire {
                v_pre: "100".to_string(),
                v_post: "99".to_string(),
                spend: "1".to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
            },
        };

        let action = Action {
            action_type: "test_action".to_string(),
            target: "test_target".to_string(),
            params: HashMap::new(),
            authority: "test_authority".to_string(),
        };

        let result = engine.execute(receipt, action, ExecutionMode::DryRun);

        assert_eq!(result.decision, Decision::Reject);
        assert!(result.execution_proof.is_none());
        assert!(result.error.is_some());
    }
}
