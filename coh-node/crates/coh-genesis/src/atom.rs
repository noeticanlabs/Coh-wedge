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
use coh_physics::measurement::SpinorProjector;

/// The GMI Atom: Computational realization of the Coh Atom
#[derive(Clone)]
pub struct GmiAtom {
    pub npe: NpeKernel,
    pub rv: RvKernel,
    pub phaseloom: PhaseLoomKernel,
    pub budgets: GlobalBudgets,
    pub ledger: crate::ledger::SimpleLedger,
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

    pub fn is_stable(&self, prev_v: u128, next_v: u128, spend: u128, defect: u128) -> bool {
        match (next_v.checked_add(spend), prev_v.checked_add(defect)) {
            (Some(lhs), Some(rhs)) => lhs <= rhs,
            _ => false,
        }
    }

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

        // 1. Envelopes
        if !self.budgets.env.hardware_available || self.budgets.env.wallclock_ms == 0 {
            return (false, trace.with_halt("Env breach"));
        }
        if !self.budgets.system.halt_available || self.budgets.system.logging_ops < 10 {
            return (false, trace.with_halt("System reserve threatened"));
        }

        // 2. Causal Cone
        let cone_check = match classify_gmi_interval(distance, c_g, dt_g) {
            Ok(c) => c,
            Err(e) => return (false, trace.with_reject(&format!("Causal error: {:?}", e))),
        };
        if cone_check.class == CausalClass::Spacelike {
            return (false, trace.with_reject("Spacelike violation"));
        }

        // 3. Spinor Pre-Calculation (Spin-Coh Atom v0.2)
        let mut next_spinor = None;
        let mut born_weight = None;
        if let Some(ref psi) = self.carrier {
            // Mock selection of component 0 for this prototype
            let projector = SpinorProjector::coordinate(0);
            
            // RV Gate: Matrix validation
            if !projector.validate(1e-10) {
                return (false, trace.with_reject("Invalid projector (idempotency/hermiticity failure)"));
            }

            born_weight = Some(projector.born_weight(psi));
            next_spinor = projector.measurement_update(psi);
            
            if next_spinor.is_none() {
                return (false, trace.with_reject("Zero-norm branch continuation"));
            }
            trace.events.push(format!("Spinor PRE-CALC: Lüders update ready (Weight: {:.4})", born_weight.unwrap()));
        }

        // 4. Valuation Law 1
        let prev_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);

        // 5. NPE & RV (Imagination & Authority)
        if !self.npe.is_affordable(100, 1000, 50) {
            return (false, trace.with_reject("NPE budget exhausted"));
        }
        trace.events.push("NPE PROPOSE: Candidate knowledge formed".into());
        
        let rv_cost = coh_core::rv_kernel::RvCost::default();
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
            return (false, trace.with_reject(&format!("RV failed: {:?}", decision.kind)));
        }

        // 6. Global Stability
        let next_v = self.rv.state.valuation + self.npe.governing_state.disorder + (self.phaseloom.state.tension as u128);
        if !self.is_stable(prev_v, next_v, 10, 100) {
            return (false, trace.with_reject("Global stability violation"));
        }
        trace.events.push("Atom CHECK: Global stability verified".into());

        // --- ATOMIC COMMIT ---
        
        // A. Ledger (Ω_i)
        if let Err(e) = self.ledger.append(proposal_id, claim, decision) {
            return (false, trace.with_defer(&format!("Ledger failure: {}", e)));
        }
        self.budgets.system.ledger_append_ops = self.budgets.system.ledger_append_ops.saturating_sub(1);
        trace.events.push("Ledger APPEND: Receipt emitted".into());

        // B. Spinor (kappa)
        if let Some(next) = next_spinor {
            self.carrier = Some(next);
            trace.cohbit_state = CohBitState::ConditionedContinuation;
            trace.events.push("Spinor COMMIT: Lüders continuation applied".into());
        }

        // C. Memory (Phi)
        let _ = self.phaseloom.update(&BoundaryReceiptSummary::default(), &PhaseLoomConfig::default());
        trace.events.push("PhaseLoom WRITE: Memory updated".into());
        
        trace.outcome = Some(GmiStepOutcome::CommittedWithMemoryUpdate);
        trace.events.push("Atom COMMIT: CohBit emitted successfully".into());
        (true, trace)
    }
}

impl GmiStepTrace {
    fn with_reject(mut self, msg: &str) -> Self {
        self.events.push(format!("Atom REJECT: {}", msg));
        self.outcome = Some(GmiStepOutcome::Rejected(msg.to_string()));
        self.cohbit_state = CohBitState::Rejected;
        self
    }
    fn with_halt(mut self, msg: &str) -> Self {
        self.events.push(format!("Atom HALT: {}", msg));
        self.outcome = Some(GmiStepOutcome::SafeHalt(msg.to_string()));
        self
    }
    fn with_defer(mut self, msg: &str) -> Self {
        self.events.push(format!("Atom DEFER: {}", msg));
        self.outcome = Some(GmiStepOutcome::Deferred(msg.to_string()));
        self.cohbit_state = CohBitState::Deferred;
        self
    }
}
