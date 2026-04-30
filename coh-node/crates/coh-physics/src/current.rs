//! Coherence Current Ledger (J_C^mu)

use crate::gamma;
use crate::CohSpinor;
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

    /// [COH GEOMETRY] Compute effective metric coupling
    /// g_eff = g + epsilon * J*J + zeta * Rc^2 * Pi
    pub fn effective_metric_coupling(
        &self,
        g_base: [[f64; 4]; 4],
        rc: f64,
        epsilon: f64,
        zeta: f64,
    ) -> [[f64; 4]; 4] {
        let j = [self.j0, self.j1, self.j2, self.j3];
        let mut g_eff = g_base;
        
        let rc_sq = rc * rc;
        
        for mu in 0..4 {
            for nu in 0..4 {
                // Deform metric by current product and defect stress
                g_eff[mu][nu] += epsilon * j[mu] * j[nu];
                
                // Stress tensor coupling (simplified diagonal Pi)
                if mu == nu {
                    g_eff[mu][nu] += zeta * rc_sq;
                }
            }
        }
        
        g_eff
    }
}

fn compute_bilinear(bar_psi: &[Complex64; 4], g: &gamma::Matrix4, psi: &CohSpinor) -> f64 {
    let mut val = Complex64::new(0.0, 0.0);
    for (i, bar_i) in bar_psi.iter().enumerate() {
        let mut row_sum = Complex64::new(0.0, 0.0);
        for (j, psi_j) in psi.components.iter().enumerate() {
            row_sum += g[i][j] * psi_j;
        }
        val += bar_i * row_sum;
    }
    val.re // Current components are real for Dirac current
}
