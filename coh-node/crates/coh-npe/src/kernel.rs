//! NPE Kernel: Resource-bounded proposal formation.
//!
//! "NPE has imagination without authority."

use serde::{Deserialize, Serialize};
use crate::loop_engine::NpeState;
use crate::engine::{NpeProposal, NpeError};

/// G_N: NPE governing state
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NpeGoverningState {
    /// M_N: unresolved proposal complexity / disorder
    pub disorder: u128,
    /// C_N: accumulated generation cost
    pub accumulated_cost: u128,
    /// W_N: wildness / novelty pressure
    pub wildness: f64,
    /// Q_N: proposal queue depth
    pub queue_depth: usize,
    /// Phi_N: PhaseLoom memory feedback (integrated via NpeState)
    pub memory_warmth: f64,
}

/// B_N: NPE resource budget
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeBudget {
    pub cpu_ms: u64,
    pub memory_bytes: u64,
    pub latency_ms: u64,
    pub energy_mj: Option<u64>,
    pub token_budget: Option<u64>,
    pub tool_calls: u64,
    pub queue_slots: u64,
}

impl Default for NpeBudget {
    fn default() -> Self {
        Self {
            cpu_ms: 1000,
            memory_bytes: 1024 * 1024 * 100,
            latency_ms: 500,
            energy_mj: None,
            token_budget: Some(4096),
            tool_calls: 10,
            queue_slots: 50,
        }
    }
}

/// NPE Kernel: The minimum governed generator
pub struct NpeKernel {
    pub state: NpeState,
    pub governing_state: NpeGoverningState,
    pub budget: NpeBudget,
}

impl NpeKernel {
    pub fn new(state: NpeState, governing_state: NpeGoverningState, budget: NpeBudget) -> Self {
        Self {
            state,
            governing_state,
            budget,
        }
    }

    /// NPE formation law:
    /// M(p_{n+1}) + C_gen(a) + C_res(a) <= M(p_n) + D_gen(a) + A_N
    pub fn is_formation_admissible(
        &self,
        next_disorder: u128,
        gen_cost: u128,
        res_cost: u128,
        prev_disorder: u128,
        gen_defect: u128,
        authority: u128,
    ) -> bool {
        let lhs = next_disorder.checked_add(gen_cost).and_then(|v| v.checked_add(res_cost));
        let rhs = prev_disorder.checked_add(gen_defect).and_then(|v| v.checked_add(authority));
        
        match (lhs, rhs) {
            (Some(l), Some(r)) => l <= r,
            _ => false, // Overflow = inadmissible
        }
    }

    /// Check if a proposal action is NPE-affordable
    pub fn is_affordable(&self, cpu_cost: u64, mem_cost: u64, token_cost: u64) -> bool {
        self.budget.cpu_ms >= cpu_cost && 
        self.budget.memory_bytes >= mem_cost && 
        self.budget.token_budget.map_or(true, |limit| limit >= (token_cost as u64))
    }

    /// Charge the NPE budget
    pub fn charge_budget(&mut self, cpu_cost: u64, mem_cost: u64, token_cost: u64) -> Result<(), String> {
        if !self.is_affordable(cpu_cost, mem_cost, token_cost) {
            return Err("NPE budget exhausted".to_string());
        }
        self.budget.cpu_ms = self.budget.cpu_ms.saturating_sub(cpu_cost);
        self.budget.memory_bytes = self.budget.memory_bytes.saturating_sub(mem_cost);
        if let Some(ref mut limit) = self.budget.token_budget {
            *limit = limit.saturating_sub(token_cost);
        }
        Ok(())
    }
}
