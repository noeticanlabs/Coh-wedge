//! Coh Physics - Spinor & Carrier Logic
//! 
//! "The Coh Spinor is the internal orientation-current carrier 
//! that lets a Coh Atom emit quantum-compatible CohBits."

use num_complex::Complex64;
use serde::{Deserialize, Serialize};

pub mod gamma;
pub mod current;
pub mod measurement;
pub mod proofs;

/// Coh Spinor: 4-component complex state vector
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohSpinor {
    pub components: [Complex64; 4],
}

impl CohSpinor {
    pub fn new(c1: Complex64, c2: Complex64, c3: Complex64, c4: Complex64) -> Self {
        Self {
            components: [c1, c2, c3, c4],
        }
    }

    /// norm_sq = sum |c_i|^2
    pub fn norm_sq(&self) -> f64 {
        self.components.iter().map(|c| c.norm_sqr()).sum()
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
            Some(Self { components })
        }
    }

    /// rho_C = psi† psi (alias for norm_sq)
    pub fn density(&self) -> f64 {
        self.norm_sq()
    }

    /// Dirac Adjoint: bar{psi} = psi† gamma0
    pub fn adjoint(&self) -> [Complex64; 4] {
        let mut adj = [Complex64::new(0.0, 0.0); 4];
        
        // psi† gamma0
        // (c1*, c2*, c3*, c4*) * g0
        // g0 is diag(1, 1, -1, -1) in Dirac representation
        adj[0] = self.components[0].conj();
        adj[1] = self.components[1].conj();
        adj[2] = -self.components[2].conj();
        adj[3] = -self.components[3].conj();
        
        adj
    }
}
