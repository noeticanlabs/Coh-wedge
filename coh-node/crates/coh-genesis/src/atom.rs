//! Coh Atom Architecture
//! 
//! A Coh Atom is the smallest bound system that can generate, verify, 
//! receipt, remember, and continue CohBits.

use serde::{Deserialize, Serialize};
use crate::{
    NpeKernel, RvKernel, PhaseLoomKernel, EnvironmentalEnvelope, SystemReserve, GlobalBudgets,
    GmiStepTrace, GmiStepOutcome, CausalClass, classify_gmi_interval,
};
use coh_core::types::{FormalStatus, VerifierClaim, Hash32, AuthorityTag};
use coh_core::rv_kernel::RvDecisionKind;
use coh_core::cohbit::CohBitState;
use coh_npe::receipt::BoundaryReceiptSummary;
use coh_phaseloom::PhaseLoomConfig;
use num_rational::Rational64;

use coh_physics::CohSpinor;

/// The GMI Atom: Computational realization of the Coh Atom
#[derive(Clone)]
pub struct GmiAtom {
    /// NPE: Proposal cloud
    pub npe: NpeKernel,
    /// RV: Authority nucleus
    pub rv: RvKernel,
    /// PhaseLoom: Memory orbital shell
    pub phaseloom: PhaseLoomKernel,
    /// Gov_G: Binding boundary (Global envelopes)
    pub budgets: GlobalBudgets,
    /// Receipt Ledger
    pub ledger: crate::ledger::SimpleLedger,
    /// Internal orientation-current carrier (Coh Spinor)
    pub carrier: Option<CohSpinor>,
}

impl GmiAtom {
    pub fn new(
        npe: NpeKernel,
        rv: RvKernel,
        phaseloom: PhaseLoomKernel,
        budgets: GlobalBudgets,
        carrier: Option<CohSpinor>,
    ) -> Self {
        Self {
            npe,
            rv,
            phaseloom,
            budgets,
            ledger: crate::ledger::SimpleLedger::default(),
            carrier,
        }
    }

    /// Law 1: Bound-state law (Global Admissibility)
    pub fn is_stable(&self, prev_v: u128, next_v: u128, spend: u128, defect: u128) -> bool {
        match (next_v.checked_add(spend), prev_v.checked_add(defect)) {
            (Some(lhs), Some(rhs)) => lhs <= rhs,
            _ => false, // Overflow rejects
        }
    }

    /// Emit a CohBit: The process of record formation
    pub fn emit_cohbit(
        &mut self, 
        proposal_id: &str, 
        _content: &str, 
        distance: Rational64, 
        c_g: Rational64, 
        dt_g: Rational64, 
        formal_status: FormalStatus
    ) -> (bool, GmiStepTrace) {
        let mut trace = GmiStepTrace {
            step_id: proposal_id.to_string(),
            events: vec![],
            decision: None,
            outcome: None,
            cohbit_state: CohBitState::Superposed,
        };

        // 1. Level 0: Environment Check (Binding Boundary)
        if !self.budgets.env.hardware_available || self.budgets.env.wallclock_ms == 0 {
            trace.events.push("Atom HALT: Environmental envelope breach".into());
            trace.outcome = Some(GmiStepOutcome::SafeHalt("Environmental envelope breach".into()));
            return (false, trace);
        }

        // 2. Level 1: System Reserve Check
        if !self.budgets.system.halt_available || self.budgets.system.logging_ops < 10 {
            trace.events.push("Atom HALT: System reserve threatened".into());
            trace.outcome = Some(GmiStepOutcome::SafeHalt("System reserve threatened".into()));
            return (false, trace);
        }

        // 3. Level 2: RV Reserve Check (Pre-check)
        let rv_cost = coh_core::rv_kernel::RvCost::default();
        if !self.rv.can_verify_safely(&rv_cost) {
            trace.events.push("Atom REJECT: RV reserve protection breach".into());
            trace.outcome = Some(GmiStepOutcome::Rejected("RV reserve protection breach".into()));
            return (false, trace);
        }

        // 3.5. Causal Cone Check (Spacelike Rejection)
        let cone_check = match classify_gmi_interval(distance, c_g, dt_g) {
            Ok(check) => check,
            Err(e) => {
                trace.events.push(format!("Atom REJECT: Invalid causal parameters: {:?}", e));
                trace.outcome = Some(GmiStepOutcome::Rejected(format!("Invalid causal parameters: {:?}", e)));
                return (false, trace);
            }
        };
        
        if cone_check.class == CausalClass::Spacelike {
            trace.events.push("Atom REJECT: Spacelike causal violation (d_G > c_G * dt_G)".into());
            trace.outcome = Some(GmiStepOutcome::Rejected("Spacelike causal violation".into()));
            return (false, trace);
        }

        // [LORENTZ] Calculate the Gamma factor
        let gamma = if cone_check.class == CausalClass::Null {
            100.0 // Cap at boundary
        } else {
            let cg_dt = c_g * dt_g;
            let cg_dt_f = *cg_dt.numer() as f64 / *cg_dt.denom() as f64;
            let ds2_f = *cone_check.interval_sq.numer() as f64 / *cone_check.interval_sq.denom() as f64;
            if ds2_f <= 0.0 {
                100.0
            } else {
                cg_dt_f / ds2_f.sqrt()
            }
        };

        // 4. Pre-state valuation (Law 1 prep)
        let prev_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);

        // 5. PhaseLoom Read (Bias)
        let _bias = match self.phaseloom.get_bias() {
            Ok(b) => {
                trace.events.push("PhaseLoom READ: StrategyBias emitted".into());
                b
            },
            Err(e) => {
                trace.events.push(format!("PhaseLoom REJECT: {}", e));
                trace.outcome = Some(GmiStepOutcome::Rejected(format!("PhaseLoom bias failure: {}", e)));
                return (false, trace);
            }
        };

        // 6. Level 3: NPE Propose (Budget Check)
        if !self.npe.is_affordable(100, 1000, 50) {
            trace.events.push("NPE REJECT: Budget exhausted".into());
            trace.outcome = Some(GmiStepOutcome::Rejected("NPE budget exhausted".into()));
            return (false, trace);
        }
        trace.events.push("NPE PROPOSE: Candidate knowledge formed".into());

        // 7. RV Verify (Structured Claim)
        let claim = VerifierClaim {
            claim_id: proposal_id.to_string(),
            payload_hash: Hash32::default(),
            formal_status,
            authority: AuthorityTag::RvCertification,
            law_margin: None,
        };

        let decision = self.rv.verify_claim(&claim, &rv_cost);
        trace.decision = Some(decision.kind);
        trace.events.push(format!("RV VERIFY: Decision {:?}", decision.kind));

        if decision.kind != RvDecisionKind::Accept {
            trace.outcome = Some(GmiStepOutcome::Rejected(format!("RV verification failed: {:?}", decision.kind)));
            return (false, trace);
        }

        // 9. Ledger Check (Hard Gate)
        if self.budgets.system.ledger_append_ops == 0 {
            trace.events.push("Atom REJECT: Ledger capacity exhausted".into());
            trace.outcome = Some(GmiStepOutcome::Rejected("Ledger capacity exhausted".into()));
            return (false, trace);
        }

        // 10. Post-state valuation (Projected)
        let next_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);

        // 11. Governor Global Admissibility Check (Law 1: Stability)
        if !self.is_stable(prev_v, next_v, 10, 100) {
            trace.events.push("Atom REJECT: Global stability violation".into());
            trace.outcome = Some(GmiStepOutcome::Rejected("Global stability violation".into()));
            return (false, trace);
        }

        // --- COMMIT PHASE ---
        
        // A. Ledger Append (Receipt Ω_y)
        if let Err(e) = self.ledger.append(proposal_id, claim, decision) {
            trace.events.push(format!("Atom ROLLBACK: Ledger append failed: {}", e));
            trace.outcome = Some(GmiStepOutcome::Deferred(format!("Ledger failure: {}", e)));
            return (false, trace);
        }
        
        self.budgets.system.ledger_append_ops = self.budgets.system.ledger_append_ops.checked_sub(1).unwrap_or(0);
        trace.events.push("Ledger APPEND: Receipt emitted".into());

        // B. Budget Charging
        if let Err(e) = self.npe.charge_budget(100, 1000, 50) {
            trace.events.push(format!("Atom WARNING: NPE budget charge failed: {}", e));
        }

        // C. PhaseLoom Write (Feedback Γ)
        let receipt = BoundaryReceiptSummary {
            target: proposal_id.to_string(),
            domain: "system".into(),
            accepted: true,
            outcome: "accepted".into(),
            gamma,
            ..Default::default()
        };
        
        match self.phaseloom.update(&receipt, &PhaseLoomConfig::default()) {
            Ok(_) => {
                trace.events.push("PhaseLoom WRITE: Memory updated from receipt".into());
                trace.outcome = Some(GmiStepOutcome::CommittedWithMemoryUpdate);
            },
            Err(e) => {
                trace.events.push(format!("PhaseLoom WRITE SKIPPED: {}", e));
                trace.outcome = Some(GmiStepOutcome::CommittedMemorySkipped(e));
            }
        }

        // D. Spinor Update: If a carrier exists, apply Lüders continuation
        if let Some(ref mut psi) = self.carrier {
            // In a real run, the branch would be selected by the measurement channel
            // For this prototype, we simulate a projection on component 0
            let projector = coh_physics::measurement::SpinorProjector { component_index: 0 };
            
            // RV Gate: Validate projector
            if !projector.validate() {
                trace.events.push("Spinor REJECT: Invalid projector".into());
                return (false, GmiStepTrace {
                    step_id: proposal_id.to_string(),
                    events: trace.events.clone(),
                    decision: Some(RvDecisionKind::Reject),
                    outcome: Some(GmiStepOutcome::Rejected("Invalid projector".into())),
                    cohbit_state: CohBitState::Rejected,
                });
            }

            if let Some(next_psi) = projector.measurement_update(psi) {
                trace.events.push(format!("Spinor UPDATE: Lüders continuation applied (Born Weight: {:.4})", projector.born_weight(psi)));
                *psi = next_psi;
                trace.cohbit_state = CohBitState::ConditionedContinuation;
            } else {
                // Branch norm zero: Rejection is necessary
                trace.events.push("Spinor REJECT: Zero-norm branch continuation".into());
                return (false, GmiStepTrace {
                    step_id: proposal_id.to_string(),
                    events: trace.events.clone(),
                    decision: Some(RvDecisionKind::Reject),
                    outcome: Some(GmiStepOutcome::Rejected("Zero-norm branch".into())),
                    cohbit_state: CohBitState::Rejected,
                });
            }
        }

        trace.events.push("Atom COMMIT: CohBit emitted successfully".into());
        (true, trace)
    }
}
