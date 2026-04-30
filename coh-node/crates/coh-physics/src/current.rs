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
        
        // J^mu = bar{psi} gamma^mu psi
        let j0 = compute_bilinear(&bar_psi, &gamma::gamma0(), psi);
        let j1 = compute_bilinear(&bar_psi, &gamma::gamma1(), psi);
        let j2 = compute_bilinear(&bar_psi, &gamma::gamma2(), psi);
        let j3 = compute_bilinear(&bar_psi, &gamma::gamma3(), psi);
        
        Self { j0, j1, j2, j3 }
    }
}

fn compute_bilinear(bar_psi: &[Complex64; 4], g: &gamma::Matrix4, psi: &CohSpinor) -> f64 {
    let mut val = Complex64::new(0.0, 0.0);
    for i in 0..4 {
        let mut row_sum = Complex64::new(0.0, 0.0);
        for j in 0..4 {
            row_sum += g[i][j] * psi.components[j];
        }
        val += bar_psi[i] * row_sum;
    }
    val.re // Current components are real for Dirac current
}
