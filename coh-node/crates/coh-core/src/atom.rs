use crate::cohbit::CohBit;
use crate::types::Hash32;
use serde::{Deserialize, Serialize};
use num_rational::Rational64;
use num_traits::ToPrimitive;

/// Coh Atom Geometry Metrics
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AtomGeometry {
    pub distance: Rational64,
    pub curvature: f64,
    pub ricci_scalar: f64,
}

/// Coh Atom Metabolism
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AtomMetabolism {
    pub budget: Rational64,
    pub refresh: Rational64,
}

/// The Coh Atom: Verifier-governed unit of state evolution.
/// 
/// \boxed{\mathcal A(x) = (x, \mathcal B_x, \mathcal A_x, \mathcal P_x, \mathcal G_x, \mathcal M_x, \mathcal R_x)}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CohAtom {
    pub state_hash: Hash32,
    pub valuation: Rational64,
    
    pub admissible_bits: Vec<CohBit>,
    
    pub geometry: AtomGeometry,
    pub metabolism: AtomMetabolism,
    pub receipt_chain: Vec<Hash32>,
}

impl CohAtom {
    /// Action Functional: J(e) = delta_hat(e) + F_exec(x') + lambda * R_coh(x') - U_refresh(x', t)
    /// Now includes gauge curvature (constraint conflict metric).
    pub fn compute_action(&self, bit: &CohBit, lambda: f64, gauge_curvature: f64) -> f64 {
        let delta_hat = bit.delta_hat.to_f64().unwrap_or(0.0);
        let f_exec = bit.utility; // Simplified: utility as a proxy for free energy contribution
        let ricci = self.geometry.ricci_scalar;
        
        let r_coh = ricci + gauge_curvature;
        let u_refresh = self.metabolism.refresh.to_f64().unwrap_or(0.0);
        let ym_energy = bit.ym_energy;
        
        delta_hat - f_exec + (lambda * r_coh) + ym_energy - u_refresh
    }

    /// Optimal Executable CohBit: \pi^*(x) = arg min_{e \in A_x} J(e)
    pub fn select_optimal_bit(&self, lambda: f64, gauge_curvature: f64) -> Option<&CohBit> {
        self.admissible_bits
            .iter()
            .filter(|b| b.is_executable())
            .min_by(|a, b| {
                let ja = self.compute_action(a, lambda, gauge_curvature);
                let jb = self.compute_action(b, lambda, gauge_curvature);
                ja.partial_cmp(&jb).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Metabolic Law: B_{t+1} = B_t + A_t - Spend(e_t)
    pub fn update_metabolism(&mut self, bit: &CohBit) {
        self.metabolism.budget = self.metabolism.budget + self.metabolism.refresh - bit.spend;
    }

    /// Atom Evolution: A(x) -> A(x') via bit b_i
    pub fn evolve(&mut self, bit: &CohBit, _lambda: f64, _gauge_curvature: f64) -> bool {
        if !bit.is_executable() {
            return false;
        }
        
        // V(x_{t+1}) + Spend(e_t) <= V(x_t) + Defect(e_t) + A_t
        let lhs = bit.valuation_post + bit.spend;
        let rhs = self.valuation + bit.defect + self.metabolism.refresh;
        
        if lhs > rhs {
            return false; // Metabolic rejection
        }

        // Apply transition
        self.state_hash = bit.to_state;
        self.valuation = bit.valuation_post;
        self.receipt_chain.push(bit.receipt_hash);
        self.update_metabolism(bit);
        
        // Reset admissible bits for new state (to be populated by NPE)
        self.admissible_bits.clear();
        
        true
    }
}
