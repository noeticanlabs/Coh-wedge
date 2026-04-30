use coh_core::cohbit::{CohBit, CohBitLaw, CohBitState};
use coh_core::atom::{CohAtom, AtomGeometry, AtomMetabolism};
use coh_physics::CohSpinor;
use coh_physics::current::CoherenceCurrent;
use coh_physics::gauge::YangMillsCurvature;
use coh_core::types::{Hash32, Decision};
use num_rational::Rational64;
use num_complex::Complex64;

#[test]
fn test_layer_1_cohbit_admissibility() {
    let bit = CohBit {
        from_state: Hash32([0; 32]),
        to_state: Hash32([1; 32]),
        transition_id: "test".to_string(),
        projection_hash: Hash32([2; 32]),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(10, 1),
        defect: Rational64::new(5, 1),
        delta_hat: Rational64::new(5, 1),
        utility: 1.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([3; 32]),
        state: CohBitState::Superposed,
    };

    // m = V_pre + D - V_post - S = 100 + 5 - 90 - 10 = 5
    assert_eq!(bit.margin(), Rational64::new(5, 1));
    assert!(bit.is_executable());

    // Test soft probability normalization
    let mut bits = vec![bit.clone()];
    CohBitLaw::compute_soft_probabilities(&mut bits, 1.0, 1.0);
    assert!(bits[0].probability_soft > 0.99); // Only one bit
}

#[test]
fn test_layer_2_coh_atom_evolution() {
    let state_x = Hash32([0; 32]);
    let state_y = Hash32([1; 32]);
    
    let bit = CohBit {
        from_state: state_x,
        to_state: state_y,
        transition_id: "step_1".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        defect: Rational64::new(2, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let mut atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.1,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(10, 1),
        },
        receipt_chain: vec![],
    };

    // Evolve atom
    let success = atom.evolve(&bit, 1.0, 0.0);
    assert!(success);
    assert_eq!(atom.state_hash, state_y);
    assert_eq!(atom.valuation, Rational64::new(90, 1));
    // Budget: 1000 + 10 (refresh) - 5 (spend) = 1005
    assert_eq!(atom.metabolism.budget, Rational64::new(1005, 1));
}

#[test]
fn test_layer_3_coh_spinor_current() {
    // Construct a spinor corresponding to a Coh Atom with valuation 1.0
    let val: f64 = 1.0;
    let mut psi = CohSpinor::new(
        Complex64::new(val.sqrt(), 0.0), // psi_0 = sqrt(V)
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );
    
    // Verify J^0 = density = V
    assert!((psi.density() - val).abs() < 1e-10);
    
    let current = CoherenceCurrent::compute(&psi);
    assert!((current.j0 - val).abs() < 1e-10);
    assert_eq!(current.j1, 0.0);

    // Verify Effective Metric Coupling
    let g_base = [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, -1.0]];
    let g_eff = current.effective_metric_coupling(g_base, 0.0, 0.1, 0.0);
    // g_eff[0][0] = 1.0 + 0.1 * J0 * J0 = 1.0 + 0.1 * 1.0 * 1.0 = 1.1
    assert!((g_eff[0][0] - 1.1).abs() < 1e-10);
}

#[test]
fn test_layer_4_coh_yang_mills_curvature() {
    let mut curvature = YangMillsCurvature {
        dim: 3,
        f: [[[0.0; 8]; 4]; 4],
    };
    
    // Set some non-Abelian constraint conflict
    curvature.f[0][1][0] = 1.0; // conflict between crypto (a=0) in t-x plane
    curvature.f[0][1][1] = 2.0; // conflict in thermal (a=1)
    
    let density = curvature.action_density();
    // Sum F^2 = 1.0^2 + 2.0^2 = 5.0
    // Note: in a real trace, indices are antisymmetrized, so F_01 and F_10 would both exist.
    assert!(density >= 5.0);
}

#[test]
fn test_full_hierarchy_control_law() {
    let state_x = Hash32([0; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([1; 32]),
        transition_id: "test".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        spend: Rational64::new(0, 1),
        defect: Rational64::new(0, 1),
        delta_hat: Rational64::new(10, 1), // Geometric cost
        utility: 50.0,
        probability_soft: 0.0,
        probability_exec: 0.0,
        rv_status: Decision::Accept,
        receipt_hash: Hash32([2; 32]),
        state: CohBitState::Superposed,
    };

    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry {
            distance: Rational64::new(0, 1),
            curvature: 0.0,
            ricci_scalar: 0.5,
        },
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(5, 1),
        },
        receipt_chain: vec![],
    };

    // Action J = delta_hat + utility + lambda * (ricci + gauge) - refresh
    // J = 10 + 50 + 1.0 * (0.5 + 5.0) - 5 = 60.5
    let action = atom.compute_action(&bit, 1.0, 5.0);
    assert!((action - 60.5).abs() < 1e-10);
    
    let optimal = atom.select_optimal_bit(1.0, 5.0);
    assert!(optimal.is_some());
}
