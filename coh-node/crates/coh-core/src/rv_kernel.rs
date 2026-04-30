//! Runtime Verifier (RV) Kernel
//! 
//! "RV has authority without imagination."

use crate::types::{Hash32, Decision};
use crate::reject::RejectCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RvDecisionKind {
    Accept,
    Reject,
    Defer,
    SafeHalt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RvDecision {
    pub kind: RvDecisionKind,
    pub law_margin: Option<f64>,
    pub failure_mode: Option<String>,
    pub receipt_payload: serde_json::Value,
}

/// G_R: RV governing state
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RvGoverningState {
    /// V_R: current valuation / risk state
    pub valuation: u128,
    /// S_R: verified spend state
    pub verified_spend: u128,
    /// D_R: allowable defect / uncertainty state
    pub allowable_defect: u128,
    /// Q_R: verification queue depth
    pub queue_depth: usize,
    /// H_R: ledger hash tip / receipt state
    pub ledger_tip: Hash32,
}

/// B_R^{prot}: Protected verifier budget
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtectedRvBudget {
    pub cpu_ms: u64,
    pub memory_bytes: u64,
    pub latency_ms: u64,
    pub verification_steps: u64,
    pub ledger_ops: u64,
    pub reserve_steps_min: u64,
}

impl Default for ProtectedRvBudget {
    fn default() -> Self {
        Self {
            cpu_ms: 1000,
            memory_bytes: 1024 * 1024 * 50,
            latency_ms: 200,
            verification_steps: 1000,
            ledger_ops: 100,
            reserve_steps_min: 100,
        }
    }
}

/// RV Kernel: The minimum protected admissibility authority
pub struct RvKernel {
    pub state: RvGoverningState,
    pub budget: ProtectedRvBudget,
}

impl RvKernel {
    pub fn new(state: RvGoverningState, budget: ProtectedRvBudget) -> Self {
        Self { state, budget }
    }

    /// RV admissibility law:
    /// V(x') + Spend(r) <= V(x) + Defect(r)
    pub fn is_admissible(&self, next_valuation: u128, spend: u128, prev_valuation: u128, defect: u128) -> bool {
        let lhs = next_valuation.saturating_add(spend);
        let rhs = prev_valuation.saturating_add(defect);
        lhs <= rhs
    }

    /// Check if RV can verify safely (reserve check)
    pub fn can_verify_safely(&self, estimated_cost: u64) -> bool {
        self.budget.cpu_ms.saturating_sub(estimated_cost) >= self.budget.reserve_steps_min
    }

    /// Primary Authority Entry Point: Verify a projected claim
    pub fn verify_claim(&mut self, claim: &str) -> RvDecision {
        // 1. (Mock) Verification logic
        let accepted = true;
        
        // 2. Charge budget
        self.budget.cpu_ms = self.budget.cpu_ms.saturating_sub(10);

        RvDecision {
            kind: if accepted { RvDecisionKind::Accept } else { RvDecisionKind::Reject },
            law_margin: Some(0.42),
            failure_mode: None,
            receipt_payload: serde_json::json!({
                "claim_verified": claim,
                "timestamp": 1234567890,
            }),
        }
    }
}
