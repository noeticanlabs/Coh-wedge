//! Spinor Measurement & Record Channels

use crate::CohSpinor;
use serde::{Deserialize, Serialize};
use num_complex::Complex64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordChannel {
    pub id: String,
    pub weight: f64,
}

/// Simple Orthogonal Projector on a spinor component
pub struct SpinorProjector {
    pub component_index: usize,
}

impl SpinorProjector {
    pub fn project(&self, psi: &CohSpinor) -> CohSpinor {
        let mut components = [Complex64::new(0.0, 0.0); 4];
        components[self.component_index] = psi.components[self.component_index];
        CohSpinor { components }
    }

    pub fn born_weight(&self, psi: &CohSpinor) -> f64 {
        psi.components[self.component_index].norm_sqr()
    }

    /// Lüders update: psi -> P_i psi / ||P_i psi||
    /// Returns None if the branch norm is zero.
    pub fn measurement_update(&self, psi: &CohSpinor) -> Option<CohSpinor> {
        let projected = self.project(psi);
        projected.normalize()
    }

    /// RV Gate: Validate that the projector is lawful (P^2 = P, P† = P)
    pub fn validate(&self) -> bool {
        // Simple component projector is always idempotent and Hermitian
        self.component_index < 4
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpinorMeasurementReceipt {
    pub channel_id: String,
    pub born_weight: f64,
    pub rv_decision: String,
    pub receipt_hash: String,
}
