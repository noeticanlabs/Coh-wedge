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
        let lhs = next_disorder.saturating_add(gen_cost).saturating_add(res_cost);
        let rhs = prev_disorder.saturating_add(gen_defect).saturating_add(authority);
        lhs <= rhs
    }

    /// Check if a proposal action is NPE-affordable
    pub fn is_affordable(&self, cpu_cost: u64, mem_cost: u64, token_cost: u64) -> bool {
        self.budget.cpu_ms >= cpu_cost && 
        self.budget.memory_bytes >= mem_cost && 
        self.budget.token_budget.map_or(true, |limit| limit >= (token_cost as u64))
    }
}
