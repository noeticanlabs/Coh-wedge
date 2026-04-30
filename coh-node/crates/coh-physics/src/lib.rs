//! Coh Physics - Spinor & Carrier Logic
//! 
//! "The Coh Spinor is the minimal field representation of a Coh Atom 
//! whose conserved object is a verifier-admissible current."

use num_complex::Complex64;
use serde::{Deserialize, Serialize};

pub mod gamma;
pub mod current;
pub mod measurement;
pub mod gauge;
pub mod proofs;

/// Coh Spinor: (\Psi, \Lambda)
/// Continuous field representation of a Coh Atom.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohSpinor {
    pub components: [Complex64; 4],
    /// \Lambda: Coherence Lagrange multiplier field (enforces admissibility)
    pub lambda: f64,
    /// \nabla_\mu \Lambda: Control gauge field A_\mu (enforces \nabla J = S_C)
    pub nabla_lambda: [f64; 4],
    /// S_C: Admissible defect source (Law of Genesis in field limit)
    pub source_defect: f64,
}

impl CohSpinor {
    pub fn new(c1: Complex64, c2: Complex64, c3: Complex64, c4: Complex64) -> Self {
        Self {
            components: [c1, c2, c3, c4],
            lambda: 0.0,
            nabla_lambda: [0.0; 4],
            source_defect: 0.0,
        }
    }

    /// norm_sq = sum |c_i|^2 = J^0 (Density of Coh Atoms)
    pub fn norm_sq(&self) -> f64 {
        self.components.iter().map(|c| c.norm_sqr()).sum()
    }

    /// rho = J^0
    pub fn density(&self) -> f64 {
        self.norm_sq()
    }

    /// Normalize the spinor. Returns None if norm is zero.
    pub fn normalize(&self) -> Option<Self> {
        let n2 = self.norm_sq();
        if n2 <= 1e-15 {
            None
        } else {
            let n = n2.sqrt();
            let mut components = self.components;
            for c in components.iter_mut() {
                *c /= n;
            }
            Some(Self { 
                components,
                lambda: self.lambda,
                nabla_lambda: self.nabla_lambda,
                source_defect: self.source_defect,
            })
        }
    }

    /// Dirac Adjoint: bar{psi} = psi† gamma0
    pub fn adjoint(&self) -> [Complex64; 4] {
        let mut adj = [Complex64::new(0.0, 0.0); 4];
        adj[0] = self.components[0].conj();
        adj[1] = self.components[1].conj();
        adj[2] = -self.components[2].conj();
        adj[3] = -self.components[3].conj();
        adj
    }

    /// Observable Defect Condition: S_C = s(x) * J^0
    /// Returns the defect-per-density coefficient s(x).
    pub fn defect_coefficient(&self) -> f64 {
        let rho = self.density();
        if rho > 1e-15 {
            self.source_defect / rho
        } else {
            0.0
        }
    }

    /// Check if the Coh-Dirac Admissibility Constraint is satisfied: \nabla_\mu J^\mu = S_C
    pub fn is_admissible(&self, div_j: f64) -> bool {
        (div_j - self.source_defect).abs() < 1e-12
    }
}
