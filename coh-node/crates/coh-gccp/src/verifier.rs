use crate::state::GccpState;
use coh_core::types::MicroReceipt;
use coh_core::reject::RejectCode;

pub struct GccpVerifier {
    pub temp_limit: f64,
    pub power_limit: f64,
}

impl Default for GccpVerifier {
    fn default() -> Self {
        Self {
            temp_limit: 85.0,
            power_limit: 350.0,
        }
    }
}

impl GccpVerifier {
    pub fn verify_transition(
        &self,
        current_state: &GccpState,
        _receipt: &MicroReceipt,
    ) -> Result<(), RejectCode> {
        // 1. Basic thermal check
        if current_state.thermal.die_temp > self.temp_limit {
            return Err(RejectCode::StepBudgetExceeded); // Thermal breach
        }

        // 2. Power check
        if current_state.power.draw_watts > current_state.power.cap_watts {
            return Err(RejectCode::StepBudgetExceeded); // Power breach
        }

        // 3. Accounting law check (passed through from coh-core if already verified)
        // In a real system, we might check GCCP-specific budget consumption here.
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GccpState, ThermalState, PowerState};

    #[test]
    fn test_gccp_thermal_breach() {
        let verifier = GccpVerifier::default();
        let mut state = GccpState::default();
        state.thermal.die_temp = 90.0; // Above 85.0 limit

        // Receipt is not used in this simplified check yet
        // In a real test we would mock/build a receipt
    }
}
