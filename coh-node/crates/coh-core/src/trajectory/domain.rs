use serde::{Deserialize, Serialize};

// ============================================================================
// FINANCIAL DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FinancialStatus {
    Idle,
    Invoiced,
    ReadyToPay,
    Paid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialState {
    pub balance: u128,
    pub initial_balance: u128,
    pub status: FinancialStatus,
    pub current_invoice_amount: u128,
}

pub const COH_PRECISION: u128 = 1_000_000_000;

impl FinancialState {
    pub fn safety_margin(&self) -> u128 {
        if self.initial_balance == 0 {
            return COH_PRECISION;
        }
        (self.balance * COH_PRECISION) / self.initial_balance
    }

    pub fn alignment_index(&self) -> u128 {
        match self.status {
            FinancialStatus::Idle => 0,
            FinancialStatus::Invoiced => (COH_PRECISION * 3) / 10,
            FinancialStatus::ReadyToPay => (COH_PRECISION * 7) / 10,
            FinancialStatus::Paid => COH_PRECISION,
        }
    }

    pub fn to_metrics_tuple(&self) -> (u128, u128) {
        (self.balance, self.current_invoice_amount)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinancialAction {
    CreateInvoice { amount: u128 },
    VerifyVendor,
    IssuePayment { amount: u128 },
}

// ============================================================================
// AGENT DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Observing,
    Acting,
    PolicyReview,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub complexity_index: u64,
    pub complexity_budget: u64,
    pub authority_level: u8,
    pub status: AgentStatus,
}

impl AgentState {
    pub fn safety_margin(&self) -> u128 {
        if self.complexity_budget == 0 {
            return COH_PRECISION;
        }
        let index = self.complexity_index as u128 * COH_PRECISION;
        let budget = self.complexity_budget as u128;
        COH_PRECISION.saturating_sub(index / budget)
    }

    /// Returns a standardized target-advancement score.
    /// Note: PolicyReview is a functional setback (0.2) to reflect safety-driven restarts.
    pub fn alignment_index(&self) -> u128 {
        match self.status {
            AgentStatus::Observing => COH_PRECISION / 10,
            AgentStatus::Acting => (COH_PRECISION * 5) / 10,
            AgentStatus::PolicyReview => (COH_PRECISION * 2) / 10, // Setback for safety check
            AgentStatus::Completed => COH_PRECISION,
        }
    }

    pub fn to_metrics_tuple(&self) -> (u128, u128) {
        (self.complexity_index as u128, self.authority_level as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentAction {
    RetrieveData,
    CallTool { tool_id: String },
    UpdatePolicy,
    Finalize,
}

// ============================================================================
// OPS DOMAIN: Canonical State Machine
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OpsStatus {
    Open,
    InProgress,
    MaterialsLogged,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpsState {
    pub status: OpsStatus,
    pub materials_logged: bool,
    pub stall_risk: u64,         // Graded safety margin [0, 100]
    pub resource_readiness: u64, // Graded readiness [0, 100]
}

impl OpsState {
    pub fn safety_margin(&self) -> u128 {
        let stall_risk_fp = (self.stall_risk as u128 * COH_PRECISION) / 100;
        let readiness_fp = (self.resource_readiness as u128 * COH_PRECISION) / 100;
        COH_PRECISION
            .saturating_sub(stall_risk_fp)
            .min(readiness_fp)
    }

    pub fn alignment_index(&self) -> u128 {
        match self.status {
            OpsStatus::Open => 0,
            OpsStatus::InProgress => (COH_PRECISION * 3) / 10,
            OpsStatus::MaterialsLogged => (COH_PRECISION * 7) / 10,
            OpsStatus::Closed => COH_PRECISION,
        }
    }

    pub fn to_metrics_tuple(&self) -> (u128, u128) {
        (
            if self.materials_logged { 1 } else { 0 },
            self.status as u32 as u128,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpsAction {
    OpenWorkOrder,
    StartWork,
    LogMaterials,
    CloseTicket,
}

use crate::trajectory::types::{Action, DomainState};

/// Get admissible actions based on current semantic state
pub fn admissible_actions(state: &DomainState) -> Vec<Action> {
    match state {
        DomainState::Financial(fs) => match fs.status {
            FinancialStatus::Idle => vec![Action::Financial(FinancialAction::CreateInvoice {
                amount: fs.balance / 10, // Deterministically small: 10% of balance
            })],
            FinancialStatus::Invoiced => vec![Action::Financial(FinancialAction::VerifyVendor)],
            FinancialStatus::ReadyToPay => {
                let mut actions = vec![];
                if fs.balance >= fs.current_invoice_amount {
                    actions.push(Action::Financial(FinancialAction::IssuePayment {
                        amount: fs.current_invoice_amount,
                    }));
                }

                // Only propose inadmissible action if balance is above a risk threshold
                if fs.balance > 100 * COH_PRECISION {
                    actions.push(Action::Financial(FinancialAction::IssuePayment {
                        amount: fs.balance.saturating_add(500 * COH_PRECISION),
                    }));
                }

                actions
            }
            FinancialStatus::Paid => vec![],
        },
        DomainState::Agent(as_state) => match as_state.status {
            AgentStatus::Observing => vec![Action::Agent(AgentAction::RetrieveData)],
            AgentStatus::Acting => vec![
                Action::Agent(AgentAction::CallTool {
                    tool_id: "search".to_string(),
                }),
                Action::Agent(AgentAction::UpdatePolicy),
                Action::Agent(AgentAction::Finalize),
            ],
            AgentStatus::PolicyReview => vec![Action::Agent(AgentAction::RetrieveData)],
            AgentStatus::Completed => vec![],
        },
        DomainState::Ops(os) => match os.status {
            OpsStatus::Open => vec![Action::Ops(OpsAction::StartWork)],
            OpsStatus::InProgress => vec![Action::Ops(OpsAction::LogMaterials)],
            OpsStatus::MaterialsLogged => vec![Action::Ops(OpsAction::CloseTicket)],
            OpsStatus::Closed => vec![],
        },
    }
}

/// Derive next semantic state
pub fn derive_state(state: &DomainState, action: &Action) -> DomainState {
    match (state, action) {
        (DomainState::Financial(fs), Action::Financial(fa)) => {
            let mut next = fs.clone();
            match fa {
                FinancialAction::CreateInvoice { amount } => {
                    if fs.status == FinancialStatus::Idle {
                        next.status = FinancialStatus::Invoiced;
                        next.current_invoice_amount = *amount;
                    }
                }
                FinancialAction::VerifyVendor => {
                    if fs.status == FinancialStatus::Invoiced {
                        next.status = FinancialStatus::ReadyToPay;
                    }
                }
                FinancialAction::IssuePayment { amount } => {
                    if fs.status == FinancialStatus::ReadyToPay && fs.balance >= *amount {
                        next.status = FinancialStatus::Paid;
                        next.balance = next.balance.saturating_sub(*amount);
                    }
                }
            }
            DomainState::Financial(next)
        }
        (DomainState::Agent(as_state), Action::Agent(aa)) => {
            let mut next = as_state.clone();
            match aa {
                AgentAction::RetrieveData => {
                    next.complexity_index += 1;
                    next.status = if as_state.status == AgentStatus::PolicyReview {
                        AgentStatus::Observing
                    } else {
                        AgentStatus::Acting
                    };
                }
                AgentAction::CallTool { .. } => {
                    next.complexity_index += 2;
                }
                AgentAction::UpdatePolicy => {
                    next.authority_level += 1;
                    next.status = AgentStatus::PolicyReview;
                }
                AgentAction::Finalize => {
                    next.status = AgentStatus::Completed;
                }
            }
            DomainState::Agent(next)
        }
        (DomainState::Ops(os), Action::Ops(oa)) => {
            let mut next = os.clone();
            match oa {
                OpsAction::OpenWorkOrder => next.status = OpsStatus::Open,
                OpsAction::StartWork => next.status = OpsStatus::InProgress,
                OpsAction::LogMaterials => {
                    next.materials_logged = true;
                    next.status = OpsStatus::MaterialsLogged;
                }
                OpsAction::CloseTicket => next.status = OpsStatus::Closed,
            }
            DomainState::Ops(next)
        }
        _ => state.clone(),
    }
}

/// Explicit semantic legality check (Phase 2 of constructor rigor)
pub fn is_transition_valid_semantic(
    prev: &DomainState,
    action: &Action,
    next: &DomainState,
) -> bool {
    match (prev, action, next) {
        (DomainState::Ops(_), Action::Ops(OpsAction::CloseTicket), DomainState::Ops(os_next)) => {
            // Rule: Must have logged materials before closing
            os_next.materials_logged
        }
        (
            DomainState::Agent(_),
            Action::Agent(AgentAction::Finalize),
            DomainState::Agent(as_next),
        ) => {
            // Rule: Must be acting or at a finalized phase (avoid finalizing from observing)
            as_next.status == AgentStatus::Completed
        }
        (
            DomainState::Financial(fs_prev),
            Action::Financial(FinancialAction::IssuePayment { amount }),
            _,
        ) => {
            // Rule: Balance must cover payment
            fs_prev.balance >= *amount
        }
        _ => true, // Default to true if simple state derivative holds
    }
}

impl DomainState {
    pub fn safety_margin(&self) -> u128 {
        match self {
            DomainState::Financial(fs) => fs.safety_margin(),
            DomainState::Agent(as_state) => as_state.safety_margin(),
            DomainState::Ops(os) => os.safety_margin(),
        }
    }

    pub fn alignment_index(&self) -> u128 {
        match self {
            DomainState::Financial(fs) => fs.alignment_index(),
            DomainState::Agent(as_state) => as_state.alignment_index(),
            DomainState::Ops(os) => os.alignment_index(),
        }
    }

    pub fn to_metrics_tuple(&self) -> (u128, u128) {
        match self {
            DomainState::Financial(fs) => fs.to_metrics_tuple(),
            DomainState::Agent(as_state) => as_state.to_metrics_tuple(),
            DomainState::Ops(os) => os.to_metrics_tuple(),
        }
    }
}
