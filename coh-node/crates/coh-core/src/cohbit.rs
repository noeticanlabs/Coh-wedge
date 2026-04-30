use crate::types::{Decision, Hash32, VerifierClaim};
use serde::{Deserialize, Serialize};

/// CohBit State Space
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CohBitState {
    #[default]
    Superposed,
    Projected,
    CandidateRecord,
    RVAccepted,
    ReceiptEmitted,
    ConditionedContinuation,
    Rejected,
    Deferred,
    Remembered,
}

/// The CohBit: Minimal unit of governed information formation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohBit<X, Y> {
    pub proposal: X,
    pub projection: Y,
    pub state: CohBitState,
    pub receipt_id: Option<String>,
}

impl<X, Y> CohBit<X, Y> {
    pub fn new(proposal: X, projection: Y) -> Self {
        Self {
            proposal,
            projection,
            state: CohBitState::Superposed,
            receipt_id: None,
        }
    }

    pub fn transit(&mut self, next: CohBitState) {
        self.state = next;
    }
}

/// Quantum CohBit Specialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumCohBit {
    pub amplitude_alpha: f64, // simplified 2-branch model
    pub amplitude_beta: f64,
    pub branch_id: Option<usize>,
}

impl QuantumCohBit {
    /// Calculate Born probability for branch i
    pub fn born_probability(&self, branch: usize) -> f64 {
        match branch {
            0 => self.amplitude_alpha.powi(2),
            1 => self.amplitude_beta.powi(2),
            _ => 0.0,
        }
    }

    /// Measurement transition: proposal -> projection -> receipt -> continuation
    pub fn measure(&mut self, branch: usize) -> Decision {
        if self.born_probability(branch) > 0.0 {
            self.branch_id = Some(branch);
            Decision::Accept
        } else {
            Decision::Reject
        }
    }
}

/// GMI Atom connecting CohBit to kernels
pub struct GmiAtom {
    pub claim: VerifierClaim,
    pub cohbit: CohBit<VerifierClaim, Hash32>,
}

impl GmiAtom {
    pub fn from_claim(claim: VerifierClaim) -> Self {
        let projection = claim.payload_hash;
        Self {
            claim: claim.clone(),
            cohbit: CohBit::new(claim, projection),
        }
    }
}
