//! Rust mirrors of Lean proofs in Coh/Physics/Spinor/Proofs.lean
//!
//! Each function here computationally verifies the same statement
//! that is formally proved in Lean 4.
//!
//! Theorem correspondence table:
//!   gamma0_sq_eq_one          -> test_gamma0_sq_eq_identity
//!   proj_density_nonneg       -> test_proj_density_nonneg
//!   coord_proj_idem           -> test_coord_proj_idempotent
//!   coord_proj_hermitian      -> test_coord_proj_hermitian
//!   coord_proj_weight_sum     -> test_coord_proj_weight_sum
//!   positive_density_theorem  -> CohSpinor::density() >= 0 (always, by f64)

use crate::measurement::SpinorProjector;
use crate::{gamma, CohSpinor};
use num_complex::Complex64;

const TOL: f64 = 1e-12;

/// Mirror of: gamma0_sq_eq_one [PROVED in Lean]
/// gamma0 * gamma0 = I_4
pub fn verify_gamma0_sq_eq_identity() -> bool {
    let g0 = gamma::gamma0();
    // Multiply g0 * g0
    let mut result = [[Complex64::new(0.0, 0.0); 4]; 4];
    for (i, row) in result.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            for (k, _) in g0[i].iter().enumerate() {
                *cell += g0[i][k] * g0[k][j];
            }
        }
    }
    // Check result == I_4
    for (i, row) in result.iter().enumerate() {
        for (j, _cell) in row.iter().enumerate() {
            let expected = if i == j {
                Complex64::new(1.0, 0.0)
            } else {
                Complex64::new(0.0, 0.0)
            };
            if (result[i][j] - expected).norm() > TOL {
                return false;
            }
        }
    }
    true
}

/// Mirror of: proj_density_nonneg [PROVED in Lean]
/// For any projector P and spinor psi, density(P psi) >= 0.
pub fn verify_proj_density_nonneg(psi: &CohSpinor, projector: &SpinorProjector) -> bool {
    projector.born_weight(psi) >= 0.0
}

/// Mirror of: coord_proj_idem [PROVED in Lean]
/// P_k * P_k = P_k for coordinate projectors.
pub fn verify_coord_proj_idempotent(k: usize) -> bool {
    let p = SpinorProjector::coordinate(k);
    let psi = CohSpinor::new(
        Complex64::new(0.8, 0.0),
        Complex64::new(0.6, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );
    // P(P(psi)) == P(psi): applying twice gives same result
    let once = p.project(&psi);
    let twice = p.project(&once);
    for i in 0..4 {
        if (once.components[i] - twice.components[i]).norm() > TOL {
            return false;
        }
    }
    true
}

/// Mirror of: coord_proj_hermitian [PROVED in Lean]
/// P_k† = P_k: coordinate projectors are Hermitian.
pub fn verify_coord_proj_hermitian(k: usize) -> bool {
    let p = SpinorProjector::coordinate(k);
    // Coordinate projector matrix is real diagonal: P[i][j] = delta(i,k) * delta(j,k)
    // This is trivially Hermitian. Verify via validate():
    p.validate(TOL)
}

/// Mirror of: coord_proj_weight_sum [PROVED in Lean]
/// sum_{k=0}^{3} |P_k psi|^2 = |psi|^2 = density(psi)
pub fn verify_coord_proj_weight_sum(psi: &CohSpinor) -> bool {
    let total: f64 = (0..4)
        .map(|k| SpinorProjector::coordinate(k).born_weight(psi))
        .sum();
    (total - psi.density()).abs() < TOL
}

/// Mirror of: positive_density_theorem [PROVED in Lean]
/// density(psi) >= 0 for any CohSpinor.
pub fn verify_positive_density(psi: &CohSpinor) -> bool {
    psi.density() >= 0.0
}

/// Mirror of: cohspinor_density_eq_one [PROVED in Lean]
/// A normalized CohSpinor has density == 1.
pub fn verify_normalized_density_is_one(psi: &CohSpinor) -> bool {
    match psi.normalize() {
        Some(normalized) => (normalized.density() - 1.0).abs() < TOL,
        None => true, // Zero spinor: vacuously true
    }
}

/// J0 = density: The time-like component of the coherence current equals the density.
/// Mirror of: j0_eq_density (structural result in Lean).
pub fn verify_j0_eq_density(psi: &CohSpinor) -> bool {
    use crate::current::CoherenceCurrent;
    let current = CoherenceCurrent::compute(psi);
    (current.j0 - psi.density()).abs() < TOL
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spinor() -> CohSpinor {
        CohSpinor::new(
            Complex64::new(0.8, 0.0),
            Complex64::new(0.6, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
        )
    }

    #[test]
    fn test_gamma0_sq_eq_identity() {
        assert!(
            verify_gamma0_sq_eq_identity(),
            "gamma0^2 must equal I_4 (Lean: gamma0_sq_eq_one)"
        );
    }

    #[test]
    fn test_proj_density_nonneg() {
        let psi = test_spinor();
        for k in 0..4 {
            let p = SpinorProjector::coordinate(k);
            assert!(
                verify_proj_density_nonneg(&psi, &p),
                "Projection weight must be non-negative (Lean: proj_density_nonneg)"
            );
        }
    }

    #[test]
    fn test_coord_proj_idempotent() {
        for k in 0..4 {
            assert!(
                verify_coord_proj_idempotent(k),
                "P_k^2 = P_k must hold (Lean: coord_proj_idem)"
            );
        }
    }

    #[test]
    fn test_coord_proj_hermitian() {
        for k in 0..4 {
            assert!(
                verify_coord_proj_hermitian(k),
                "P_k† = P_k must hold (Lean: coord_proj_hermitian)"
            );
        }
    }

    #[test]
    fn test_coord_proj_weight_sum() {
        let psi = test_spinor();
        assert!(
            verify_coord_proj_weight_sum(&psi),
            "sum_k |P_k psi|^2 = density(psi) (Lean: coord_proj_weight_sum)"
        );
    }

    #[test]
    fn test_positive_density() {
        let psi = test_spinor();
        assert!(
            verify_positive_density(&psi),
            "density >= 0 (Lean: positive_density_theorem)"
        );
    }

    #[test]
    fn test_normalized_density_is_one() {
        let psi = test_spinor();
        assert!(
            verify_normalized_density_is_one(&psi),
            "normalized density == 1 (Lean: cohspinor_density_eq_one)"
        );
    }

    #[test]
    fn test_j0_eq_density() {
        let psi = test_spinor();
        assert!(
            verify_j0_eq_density(&psi),
            "J0 == density(psi) (Lean: j0_eq_density)"
        );
    }
}
