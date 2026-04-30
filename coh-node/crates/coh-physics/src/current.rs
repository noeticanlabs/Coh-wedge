//! Coherence Current Ledger (J_C^mu)

use crate::CohSpinor;
use crate::gamma;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoherenceCurrent {
    pub j0: f64,
    pub j1: f64,
    pub j2: f64,
    pub j3: f64,
}

impl CoherenceCurrent {
    pub fn compute(psi: &CohSpinor) -> Self {
        let bar_psi = psi.adjoint();
        
        // J0 = bar{psi} gamma0 psi = psi† psi (density)
        let j0 = psi.density();
        
        // J1 = bar{psi} gamma1 psi
        let g1 = gamma::gamma1();
        let mut j1_complex = Complex64::new(0.0, 0.0);
        
        for i in 0..4 {
            let mut row_sum = Complex64::new(0.0, 0.0);
            for j in 0..4 {
                row_sum += g1[i][j] * psi.components[j];
            }
            j1_complex += bar_psi[i] * row_sum;
        }
        
        // J1 must be real for a Dirac current
        let j1 = j1_complex.re;
        
        Self {
            j0,
            j1,
            j2: 0.0, // Placeholder
            j3: 0.0, // Placeholder
        }
    }
}
