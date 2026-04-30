//! Spinor Measurement & Record Channels

use crate::CohSpinor;
use serde::{Deserialize, Serialize};
use num_complex::Complex64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordChannel {
    pub id: String,
    pub weight: f64,
}

/// Orthogonal Projector on a spinor state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpinorProjector {
    pub id: String,
    pub matrix: [[Complex64; 4]; 4],
}

impl SpinorProjector {
    /// Coordinate basis component projector (Helper)
    pub fn coordinate(index: usize) -> Self {
        let mut matrix = [[Complex64::new(0.0, 0.0); 4]; 4];
        if index < 4 {
            matrix[index][index] = Complex64::new(1.0, 0.0);
        }
        Self {
            id: format!("coord_{}", index),
            matrix,
        }
    }

    pub fn project(&self, psi: &CohSpinor) -> CohSpinor {
        let mut components = [Complex64::new(0.0, 0.0); 4];
        for i in 0..4 {
            let mut sum = Complex64::new(0.0, 0.0);
            for j in 0..4 {
                sum += self.matrix[i][j] * psi.components[j];
            }
            components[i] = sum;
        }
        CohSpinor { components }
    }

    pub fn born_weight(&self, psi: &CohSpinor) -> f64 {
        let projected = self.project(psi);
        projected.norm_sq()
    }

    /// Lüders update: psi -> P_i psi / ||P_i psi||
    pub fn measurement_update(&self, psi: &CohSpinor) -> Option<CohSpinor> {
        let projected = self.project(psi);
        projected.normalize()
    }

    /// RV Gate: Validate that the projector is lawful (P^2 = P, P† = P)
    pub fn validate(&self, tolerance: f64) -> bool {
        // 1. Idempotency: P^2 = P
        for i in 0..4 {
            for j in 0..4 {
                let mut p2_ij = Complex64::new(0.0, 0.0);
                for k in 0..4 {
                    p2_ij += self.matrix[i][k] * self.matrix[k][j];
                }
                if (p2_ij - self.matrix[i][j]).norm() > tolerance {
                    return false;
                }
            }
        }

        // 2. Hermiticity: P† = P
        for i in 0..4 {
            for j in 0..4 {
                if (self.matrix[i][j] - self.matrix[j][i].conj()).norm() > tolerance {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpinorMeasurementReceipt {
    pub channel_id: String,
    pub born_weight: f64,
    pub rv_decision: String,
    pub receipt_hash: String,
}
