use crate::types::{Decision, Hash32};
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::ToPrimitive;

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
/// 
/// \boxed{ \mathfrak b_i(x) = (r_i, x_i', R_i, m_i, u_i, p_i, c_i) }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohBit {
    pub from_state: Hash32,
    pub to_state: Hash32,
    pub transition_id: String,
    pub projection_hash: Hash32,
    
    pub valuation_pre: Rational64,
    pub valuation_post: Rational64,
    pub spend: Rational64,
    pub defect: Rational64,
    pub delta_hat: Rational64, // certified_envelope
    
    pub utility: f64,
    pub probability_soft: f64,
    pub probability_exec: f64,
    
    pub rv_status: Decision,
    pub receipt_hash: Hash32,
    pub state: CohBitState,
}

impl CohBit {
    /// Admissibility Margin: m_i(x) = V(x) + D_i(x) - V(x_i') - Spend(r_i)
    pub fn margin(&self) -> Rational64 {
        self.valuation_pre + self.defect - self.valuation_post - self.spend
    }

    /// Executability Condition: m_i(x) >= 0 and RV(c_i) = ACCEPT
    pub fn is_executable(&self) -> bool {
        self.margin() >= Rational64::from_integer(0) && self.rv_status == Decision::Accept
    }

    /// Identity CohBit: \mathbf 1_x = (id, x, Pi(id), 0, 0, 1, c_x)
    pub fn identity(state_hash: Hash32, valuation: Rational64) -> Self {
        Self {
            from_state: state_hash,
            to_state: state_hash,
            transition_id: "identity".to_string(),
            projection_hash: state_hash,
            valuation_pre: valuation,
            valuation_post: valuation,
            spend: Rational64::from_integer(0),
            defect: Rational64::from_integer(0),
            delta_hat: Rational64::from_integer(0),
            utility: 0.0,
            probability_soft: 1.0,
            probability_exec: 1.0,
            rv_status: Decision::Accept,
            receipt_hash: state_hash,
            state: CohBitState::Remembered,
        }
    }

    /// Sequential Composition: \mathfrak b_j \circ \mathfrak b_i
    pub fn compose(&self, other: &Self) -> Option<Self> {
        if self.to_state != other.from_state {
            return None;
        }
        
        Some(Self {
            from_state: self.from_state,
            to_state: other.to_state,
            transition_id: format!("{}:{}", self.transition_id, other.transition_id),
            projection_hash: self.projection_hash, // Simplified: should be composite
            valuation_pre: self.valuation_pre,
            valuation_post: other.valuation_post,
            spend: self.spend + other.spend,
            defect: self.defect + other.defect,
            delta_hat: self.delta_hat + other.delta_hat,
            utility: self.utility + other.utility,
            probability_soft: self.probability_soft * other.probability_soft,
            probability_exec: self.probability_exec * other.probability_exec,
            rv_status: if self.rv_status == Decision::Accept && other.rv_status == Decision::Accept {
                Decision::Accept
            } else {
                Decision::Reject
            },
            receipt_hash: self.receipt_hash, // Simplified
            state: CohBitState::Superposed,
        })
    }

    /// Parallel Composition: \mathfrak b_a \otimes \mathfrak b_b
    pub fn parallel_compose(&self, other: &Self) -> Self {
        Self {
            from_state: self.from_state, // Composite state hash should ideally be computed
            to_state: self.to_state,
            transition_id: format!("{}+{}", self.transition_id, other.transition_id),
            projection_hash: self.projection_hash,
            valuation_pre: self.valuation_pre + other.valuation_pre,
            valuation_post: self.valuation_post + other.valuation_post,
            spend: self.spend + other.spend,
            defect: self.defect + other.defect,
            delta_hat: self.delta_hat + other.delta_hat,
            utility: self.utility + other.utility,
            probability_soft: self.probability_soft * other.probability_soft,
            probability_exec: self.probability_exec * other.probability_exec,
            rv_status: if self.rv_status == Decision::Accept && other.rv_status == Decision::Accept {
                Decision::Accept
            } else {
                Decision::Reject
            },
            receipt_hash: self.receipt_hash,
            state: CohBitState::Superposed,
        }
    }

    pub fn transit(&mut self, next: CohBitState) {
        self.state = next;
    }
}

/// CohBit Algebra and Laws
pub struct CohBitLaw;

impl CohBitLaw {
    /// Soft Proposal Law: p_i^{soft}(x) = (e^{u_i/\tau} * \sigma(\beta m_i)) / Z
    pub fn compute_soft_probabilities(bits: &mut [CohBit], tau: f64, beta: f64) {
        let weights: Vec<f64> = bits.iter().map(|b| {
            let m = b.margin().to_f64().unwrap_or(0.0);
            let gate = 1.0 / (1.0 + (-beta * m).exp()); // sigmoid gate
            (b.utility / tau).exp() * gate
        }).collect();

        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for (i, b) in bits.iter_mut().enumerate() {
                b.probability_soft = weights[i] / sum;
            }
        }
    }

    /// Hard Execution Law: p_i^{exec}(x) = (e^{u_i/\tau} * 1_{m_i>=0} * 1_{RV=ACCEPT}) / Z_exec
    pub fn compute_exec_probabilities(bits: &mut [CohBit], tau: f64) {
        let weights: Vec<f64> = bits.iter().map(|b| {
            if b.is_executable() {
                (b.utility / tau).exp()
            } else {
                0.0
            }
        }).collect();

        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for (i, b) in bits.iter_mut().enumerate() {
                b.probability_exec = weights[i] / sum;
            }
        } else {
            // Identity fallback should be handled by the caller/engine
        }
    }
}

/// Partition Functions and Entropy
pub struct CohBitThermodynamics;

impl CohBitThermodynamics {
    /// soft_entropy = -sum p_soft * log p_soft
    pub fn soft_entropy(bits: &[CohBit]) -> f64 {
        bits.iter().map(|b| {
            if b.probability_soft > 0.0 {
                -b.probability_soft * b.probability_soft.ln()
            } else {
                0.0
            }
        }).sum()
    }

    /// exec_entropy = -sum p_exec * log p_exec
    pub fn exec_entropy(bits: &[CohBit]) -> f64 {
        bits.iter().map(|b| {
            if b.probability_exec > 0.0 {
                -b.probability_exec * b.probability_exec.ln()
            } else {
                0.0
            }
        }).sum()
    }

    /// Enforcement Loss: Delta S = S_soft - S_exec
    pub fn enforcement_loss(bits: &[CohBit]) -> f64 {
        Self::soft_entropy(bits) - Self::exec_entropy(bits)
    }
}
