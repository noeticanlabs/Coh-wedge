//! PhaseLoom Kernel (PL_K)
//! 
//! "PhaseLoom has influence, but not authority."

use crate::{PhaseLoomState, PhaseLoomConfig, BoundaryReceiptSummary};
use serde::{Deserialize, Serialize};

use crate::budget::PhaseLoomBudget;

/// PhaseLoom Kernel: Governed adaptive memory system
pub struct PhaseLoomKernel {
    pub state: PhaseLoomState,
    pub budget: PhaseLoomBudget,
}

impl PhaseLoomKernel {
    pub fn new(state: PhaseLoomState, budget: PhaseLoomBudget) -> Self {
        Self { state, budget }
    }

    /// Update rule: Phi_{n+1} = mu(Phi_n, RV(r_n))
    pub fn update(&mut self, receipt: &BoundaryReceiptSummary, config: &PhaseLoomConfig) -> Result<(), String> {
        if self.budget.write_ops == 0 {
            return Err("PhaseLoom write budget exhausted".to_string());
        }
        
        self.state.ingest(receipt, config);
        self.budget.write_ops = self.budget.write_ops.saturating_sub(1);
        Ok(())
    }

    /// Read path: Phi_n -> NPE strategy bias
    pub fn get_bias(&mut self) -> Result<PhaseLoomState, String> {
        if self.budget.read_ops == 0 {
            return Err("PhaseLoom read budget exhausted".to_string());
        }
        
        self.budget.read_ops = self.budget.read_ops.saturating_sub(1);
        Ok(self.state.clone())
    }
}
