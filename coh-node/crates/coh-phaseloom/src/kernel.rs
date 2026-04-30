//! PhaseLoom Kernel (PL_K)
//! 
//! "PhaseLoom has influence, but not authority."

use crate::{PhaseLoomState, PhaseLoomConfig, BoundaryReceiptSummary};

use crate::budget::PhaseLoomBudget;

/// PhaseLoom Kernel: Governed adaptive memory system
#[derive(Clone, Debug)]
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
        // 1. Operational Check
        if self.budget.write_ops == 0 {
            return Err("PhaseLoom write budget exhausted".to_string());
        }

        // 2. Adaptive Constraints
        if self.state.tension as f64 > self.budget.max_tension {
            return Err("PhaseLoom tension limit exceeded (Critical boundary pressure)".to_string());
        }
        
        self.state.ingest(receipt, config);
        self.budget.write_ops = self.budget.write_ops.checked_sub(1).ok_or("underflow")?;
        Ok(())
    }

    /// Read path: Phi_n -> NPE strategy bias
    pub fn get_bias(&mut self) -> Result<PhaseLoomState, String> {
        if self.budget.read_ops == 0 {
            return Err("PhaseLoom read budget exhausted".to_string());
        }
        
        self.budget.read_ops = self.budget.read_ops.checked_sub(1).ok_or("underflow")?;
        Ok(self.state.clone())
    }
}
