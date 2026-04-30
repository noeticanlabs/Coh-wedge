//! Gamma Matrices (Dirac Representation)

use num_complex::Complex64;

pub type Matrix4 = [[Complex64; 4]; 4];

/// Gamma 0: diag(1, 1, -1, -1)
pub fn gamma0() -> Matrix4 {
    let zero = Complex64::new(0.0, 0.0);
    let one = Complex64::new(1.0, 0.0);
    let neg_one = Complex64::new(-1.0, 0.0);
    
    [
        [one, zero, zero, zero],
        [zero, one, zero, zero],
        [zero, zero, neg_one, zero],
        [zero, zero, zero, neg_one],
    ]
}

/// Gamma 1: standard Dirac gamma1
pub fn gamma1() -> Matrix4 {
    let zero = Complex64::new(0.0, 0.0);
    let one = Complex64::new(1.0, 0.0);
    let neg_one = Complex64::new(-1.0, 0.0);
    
    [
        [zero, zero, zero, one],
        [zero, zero, one, zero],
        [zero, neg_one, zero, zero],
        [neg_one, zero, zero, zero],
    ]
}

/// Gamma 2: standard Dirac gamma2
pub fn gamma2() -> Matrix4 {
    let zero = Complex64::new(0.0, 0.0);
    let i = Complex64::new(0.0, 1.0);
    let neg_i = Complex64::new(0.0, -1.0);
    
    [
        [zero, zero, zero, neg_i],
        [zero, zero, i, zero],
        [zero, i, zero, zero],
        [neg_i, zero, zero, zero],
    ]
}

/// Gamma 3: standard Dirac gamma3
pub fn gamma3() -> Matrix4 {
    let zero = Complex64::new(0.0, 0.0);
    let one = Complex64::new(1.0, 0.0);
    let neg_one = Complex64::new(-1.0, 0.0);
    
    [
        [zero, zero, one, zero],
        [zero, zero, zero, neg_one],
        [neg_one, zero, zero, zero],
        [zero, one, zero, zero],
    ]
}
