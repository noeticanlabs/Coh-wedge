//! Coh Yang-Mills - Non-Abelian Gauge & Curvature
//!
//! "Curvature is the failure of constraints to commute. 
//! High curvature marks regions of intense constraint conflict."

use serde::{Deserialize, Serialize};
use std::ops::Add;

/// Gauge Group Trait: Defines the algebra and group operations for a constraint channel.
pub trait GaugeGroup {
    type Algebra: Clone + Default + Add<Output = Self::Algebra> + Send + Sync;
    
    /// Lie Bracket: [A, B]
    fn bracket(a: &Self::Algebra, b: &Self::Algebra) -> Self::Algebra;
    
    /// Trace: Tr(X * Y)
    fn trace_product(a: &Self::Algebra, b: &Self::Algebra) -> f64;

    /// Norm: sqrt(Tr(X^2))
    fn norm(a: &Self::Algebra) -> f64 {
        Self::trace_product(a, a).sqrt()
    }
}

/// A 2x2 complex matrix for SU(2) computations
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Su2Matrix(pub [num_complex::Complex64; 4]); // [00, 01, 10, 11]

impl Su2Matrix {
    pub fn identity() -> Self {
        Self([
            num_complex::Complex64::new(1.0, 0.0), num_complex::Complex64::new(0.0, 0.0),
            num_complex::Complex64::new(0.0, 0.0), num_complex::Complex64::new(1.0, 0.0)
        ])
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Self([
            self.0[0]*rhs.0[0] + self.0[1]*rhs.0[2], self.0[0]*rhs.0[1] + self.0[1]*rhs.0[3],
            self.0[2]*rhs.0[0] + self.0[3]*rhs.0[2], self.0[2]*rhs.0[1] + self.0[3]*rhs.0[3]
        ])
    }

    pub fn trace(&self) -> num_complex::Complex64 {
        self.0[0] + self.0[3]
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PauliAlgebra(pub [f64; 3]);

impl PauliAlgebra {
    /// Convert Pauli components to i * A_mu^a * sigma^a (the exponent term)
    pub fn to_infinitesimal_su2(&self, dt: f64) -> Su2Matrix {
        // sigma_1 = [0 1; 1 0], sigma_2 = [0 -i; i 0], sigma_3 = [1 0; 0 -1]
        // i * dt * (a1*s1 + a2*s2 + a3*s3)
        let c_i = num_complex::Complex64::new(0.0, 1.0);
        let a1 = self.0[0] * dt;
        let a2 = self.0[1] * dt;
        let a3 = self.0[2] * dt;

        let m00 = c_i * a3;
        let m01 = c_i * (num_complex::Complex64::new(a1, -a2));
        let m10 = c_i * (num_complex::Complex64::new(a1, a2));
        let m11 = c_i * (-a3);

        Su2Matrix([m00, m01, m10, m11])
    }
}

impl std::ops::Add for PauliAlgebra {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1], self.0[2] + rhs.0[2]])
    }
}

/// SU(2) Implementation: The default non-Abelian verifier group.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SU2;

impl GaugeGroup for SU2 {
    type Algebra = PauliAlgebra; // Pauli components (a=1,2,3)

    fn bracket(a: &Self::Algebra, b: &Self::Algebra) -> Self::Algebra {
        // [T^a, T^b] = i epsilon^abc T^c
        // For Pauli matrices, this is the cross product
        PauliAlgebra([
            a.0[1] * b.0[2] - a.0[2] * b.0[1],
            a.0[2] * b.0[0] - a.0[0] * b.0[2],
            a.0[0] * b.0[1] - a.0[1] * b.0[0],
        ])
    }

    fn trace_product(a: &Self::Algebra, b: &Self::Algebra) -> f64 {
        // Tr(T^a T^b) = 1/2 delta^ab
        0.5 * (a.0[0] * b.0[0] + a.0[1] * b.0[1] + a.0[2] * b.0[2])
    }
}

/// Coh Gauge Field: A_\mu^a
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohGaugeField {
    pub dim: usize,
    /// Connection components: [mu][a]
    pub connection: [[f64; 8]; 4], // Support up to 8 generators (e.g. SU(3))
}

impl CohGaugeField {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            connection: [[0.0; 8]; 4],
        }
    }

    /// Compute Yang-Mills Curvature: F_munu^a = d_mu A_nu - d_nu A_mu + [A_mu, A_nu]
    pub fn compute_curvature(&self, mu: usize, nu: usize) -> [f64; 3] {
        let a_mu = PauliAlgebra([self.connection[mu][0], self.connection[mu][1], self.connection[mu][2]]);
        let a_nu = PauliAlgebra([self.connection[nu][0], self.connection[nu][1], self.connection[nu][2]]);
        
        // Non-Abelian term: [A_mu, A_nu]
        let bracket = SU2::bracket(&a_mu, &a_nu);
        
        // Simplified derivative term: difference of components (lattice approx)
        [
            a_nu.0[0] - a_mu.0[0] + bracket.0[0],
            a_nu.0[1] - a_mu.0[1] + bracket.0[1],
            a_nu.0[2] - a_mu.0[2] + bracket.0[2],
        ]
    }
}

/// Yang-Mills Curvature Tensor: F_munu
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YangMillsCurvature {
    pub dim: usize,
    pub f: [[[f64; 8]; 4]; 4], // [mu][nu][a]
}

impl YangMillsCurvature {
    /// Compute Tr(F^2) = Tr(F_munu F^munu)
    pub fn action_density(&self) -> f64 {
        let mut sum = 0.0;
        for mu in 0..4 {
            for nu in 0..4 {
                let f_munu = PauliAlgebra([self.f[mu][nu][0], self.f[mu][nu][1], self.f[mu][nu][2]]);
                sum += SU2::trace_product(&f_munu, &f_munu);
            }
        }
        sum
    }

    /// Bianchi Identity Residual: ||D_[lambda F_munu]||
    pub fn bianchi_residual(&self) -> f64 {
        // Structural lock check: D_lambda F_munu + cyclic = 0
        0.0 // Placeholder: In a real lattice this checks cyclic sum of plaquettes
    }
}

/// Wilson Loop Receipt: Order-sensitive path-ordered receipt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WilsonLoopReceipt {
    pub path_hash: String,
    pub holonomy_trace: f64,
    pub curvature_sum: f64,
    pub constraint_residual: f64,
    pub bianchi_residual: f64,
    pub ym_energy: f64,
}

impl WilsonLoopReceipt {
    /// Compute Holonomy: W = Tr(P exp(i int A))
    pub fn compute_holonomy(history: &coh_core::trajectory::path_integral::CohHistory, gauge: &CohGaugeField) -> f64 {
        let mut w = Su2Matrix::identity();
        let dt = 0.01; // Integration step
        
        // Path-ordered product: W = U_n * U_{n-1} * ... * U_1
        for _bit in &history.steps {
            // Compute A_mu at this point (simplified for fixed field)
            let a_mu = PauliAlgebra([gauge.connection[0][0], gauge.connection[0][1], gauge.connection[0][2]]);
            
            // Infinitesimal propagator: U = exp(i A dx) approx I + i A dx
            let infinitesimal = a_mu.to_infinitesimal_su2(dt);
            let mut u = Su2Matrix::identity();
            for i in 0..4 { u.0[i] += infinitesimal.0[i]; }
            
            w = w.mul(&u);
        }
        
        w.trace().re
    }

    /// Is Holonomy Admissible: |2 - W| < epsilon (for N=2)
    pub fn is_admissible(&self, tolerance: f64) -> bool {
        (2.0 - self.holonomy_trace).abs() < tolerance
    }
}
