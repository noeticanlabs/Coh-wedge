//! Runtime Verifier (RV) Kernel
//! 
//! "RV has authority without imagination."

use crate::types::{Hash32, ToolAuthorityMode, VerifierClaim, FormalStatus};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RvCost {
    pub cpu_ms: u64,
    pub memory_bytes: u64,
    pub verification_steps: u64,
    pub ledger_ops: u64,
}

impl Default for RvCost {
    fn default() -> Self {
        Self {
            cpu_ms: 10,
            memory_bytes: 1024 * 1024,
            verification_steps: 1,
            ledger_ops: 1,
        }
    }
}

/// RV Kernel: The minimum protected admissibility authority
#[derive(Clone, Debug)]
pub struct RvKernel {
    pub state: RvGoverningState,
    pub budget: ProtectedRvBudget,
    pub mode: ToolAuthorityMode,
}

impl RvKernel {
    pub fn new(state: RvGoverningState, budget: ProtectedRvBudget) -> Self {
        Self { state, budget, mode: ToolAuthorityMode::Certification }
    }

    /// RV admissibility law:
    /// V(x') + Spend(r) <= V(x) + Defect(r)
    pub fn is_admissible(&self, next_valuation: u128, spend: u128, prev_valuation: u128, defect: u128) -> bool {
        match (next_valuation.checked_add(spend), prev_valuation.checked_add(defect)) {
            (Some(lhs), Some(rhs)) => lhs <= rhs,
            _ => false, // Overflow = inadmissible
        }
    }

    /// Check if RV can verify safely (reserve check)
    pub fn can_verify_safely(&self, cost: &RvCost) -> bool {
        self.budget.cpu_ms >= cost.cpu_ms.saturating_add(self.budget.reserve_steps_min) &&
        self.budget.verification_steps >= cost.verification_steps.saturating_add(self.budget.reserve_steps_min) &&
        self.budget.ledger_ops >= cost.ledger_ops
    }

    /// Charge the verifier budget
    pub fn charge_budget(&mut self, cost: &RvCost) -> Result<(), String> {
        if !self.can_verify_safely(cost) {
            return Err("RV budget reserve protection triggered".to_string());
        }
        self.budget.cpu_ms = self.budget.cpu_ms.saturating_sub(cost.cpu_ms);
        self.budget.verification_steps = self.budget.verification_steps.saturating_sub(cost.verification_steps);
        self.budget.ledger_ops = self.budget.ledger_ops.saturating_sub(cost.ledger_ops);
        Ok(())
    }

    /// Primary Authority Entry Point: Verify a projected claim
    pub fn verify_claim(&mut self, claim: &VerifierClaim, cost: &RvCost) -> RvDecision {
        // 1. Budget hard gate (Evaluation Spend)
        if let Err(e) = self.charge_budget(cost) {
            return RvDecision {
                kind: RvDecisionKind::Defer,
                law_margin: None,
                failure_mode: Some(e),
                receipt_payload: serde_json::json!({ "reason": "budget_exhausted" }),
            };
        }

        // 2. Formal Status Gate
        match claim.formal_status {
            FormalStatus::ProofCertified | FormalStatus::ClosedNoSorry => {
                // Proceed
            }
            _ => {
                return RvDecision {
                    kind: RvDecisionKind::Reject,
                    law_margin: Some(-1.0),
                    failure_mode: Some(format!("RV REJECT: Incomplete formal status {:?}", claim.formal_status)),
                    receipt_payload: serde_json::json!({ "status": claim.formal_status }),
                };
            }
        }

        // 3. Logic Gate
        if claim.claim_id.is_empty() {
             return RvDecision {
                kind: RvDecisionKind::Reject,
                law_margin: Some(-1.0),
                failure_mode: Some("RV REJECT: Empty claim ID".into()),
                receipt_payload: serde_json::json!({}),
            };
        }
        
        RvDecision {
            kind: RvDecisionKind::Accept,
            law_margin: Some(1.0),
            failure_mode: None,
            receipt_payload: serde_json::json!({
                "claim_id": claim.claim_id,
                "status": claim.formal_status,
                "cost": cost,
            }),
        }
    }
}
