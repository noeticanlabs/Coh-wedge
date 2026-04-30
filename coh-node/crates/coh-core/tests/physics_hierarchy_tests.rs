use coh_core::cohbit::{CohBit, CohBitLaw, CohBitState};
use coh_core::atom::{CohAtom, AtomGeometry, AtomMetabolism};
use coh_physics::CohSpinor;
use coh_physics::current::CoherenceCurrent;
use coh_physics::gauge::{CohGaugeField, YangMillsCurvature, WilsonLoopReceipt, GaugeGroup, SU2};
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
        ym_energy: 0.0,
        constraint_residual: 0.0,
        bianchi_residual: 0.0,
    };

    assert_eq!(bit.margin(), Rational64::new(5, 1));
    assert!(bit.is_executable());
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
        ym_energy: 0.0,
        constraint_residual: 0.0,
        bianchi_residual: 0.0,
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

    let success = atom.evolve(&bit, 1.0, 0.0);
    assert!(success);
    assert_eq!(atom.state_hash, state_y);
    assert_eq!(atom.valuation, Rational64::new(90, 1));
    assert_eq!(atom.metabolism.budget, Rational64::new(1005, 1));
}

#[test]
fn test_layer_3_coh_spinor_current() {
    let val: f64 = 1.0;
    let psi = CohSpinor::new(
        Complex64::new(val.sqrt(), 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );
    
    assert!((psi.density() - val).abs() < 1e-10);
    
    let current = CoherenceCurrent::compute(&psi);
    assert!((current.j0 - val).abs() < 1e-10);

    let g_base = [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, -1.0]];
    let g_eff = current.effective_metric_coupling(g_base, 0.0, 0.1, 0.0);
    assert!((g_eff[0][0] - 1.1).abs() < 1e-10);
}

#[test]
fn test_layer_4_locked_yang_mills_curvature() {
    let mut gauge = CohGaugeField::new(3);
    
    // Set non-commuting links: A_t in Pauli-x, A_x in Pauli-y
    gauge.connection[0][0] = 0.1; // A_0^1
    gauge.connection[1][1] = 0.1; // A_1^2
    
    let f01 = gauge.compute_curvature(0, 1);
    
    // Non-Abelian term: [0.1 Tx, 0.1 Ty] = 0.01 i Tz
    // f01[2] should include this bracket term
    assert!(f01[2].abs() > 0.005);
    
    let mut curvature = YangMillsCurvature {
        dim: 3,
        f: [[[0.0; 8]; 4]; 4],
    };
    curvature.f[0][1][2] = f01[2];
    
    let density = curvature.action_density();
    // Tr(F^2) = Tr( (f Tx)^2 ) = 1/2 f^2
    assert!(density > 0.0);
}

#[test]
fn test_layer_5_multi_atom_field_coupling() {
    use coh_physics::field::CohField;
    let mut field = CohField::new(0.01);
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
        delta_hat: Rational64::new(0, 1),
        utility: 10.0,
        ..Default::default()
    };

    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![bit.clone()],
        geometry: AtomGeometry::default(),
        metabolism: AtomMetabolism {
            budget: Rational64::new(1000, 1),
            refresh: Rational64::new(0, 1),
        },
        receipt_chain: vec![],
    };

    field.atoms.push(atom.clone());
    let mut atom2 = atom.clone();
    atom2.state_hash = Hash32([1; 32]);
    field.atoms.push(atom2);

    let cost = field.interaction_cost(&field.atoms[0], &bit);
    assert!((cost - 1.0).abs() < 1e-10);
}

#[test]
fn test_layer_6_path_integral_weighting() {
    use coh_core::trajectory::path_integral::CohHistory;
    let state_x = Hash32([0; 32]);
    let bit = CohBit {
        from_state: state_x,
        to_state: Hash32([1; 32]),
        transition_id: "step1".to_string(),
        projection_hash: state_x,
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(90, 1),
        spend: Rational64::new(5, 1),
        delta_hat: Rational64::new(2, 1),
        utility: 10.0,
        rv_status: Decision::Accept,
        ..Default::default()
    };

    let history = CohHistory { steps: vec![bit.clone()] };
    let atom = CohAtom {
        state_hash: state_x,
        valuation: Rational64::new(100, 1),
        admissible_bits: vec![],
        geometry: AtomGeometry::default(),
        metabolism: AtomMetabolism::default(),
        receipt_chain: vec![],
    };

    let prob = history.path_probability(&atom, 1.0, 0.0, 1.0, 1.0);
    assert!(prob > 0.0);
}

#[test]
fn test_layer_7_wilson_loop_holonomy() {
    use coh_physics::gauge::{CohGaugeField, WilsonLoopReceipt};
    use coh_core::trajectory::path_integral::CohHistory;
    
    let mut gauge = CohGaugeField::new(3);
    gauge.connection[0][0] = 0.1; // Total rotation phase
    
    let bit = CohBit {
        from_state: Hash32([0; 32]),
        to_state: Hash32([1; 32]),
        transition_id: "step".to_string(),
        valuation_pre: Rational64::new(100, 1),
        valuation_post: Rational64::new(100, 1),
        rv_status: Decision::Accept,
        ..Default::default()
    };

    let history = CohHistory { steps: vec![bit.clone(); 10] };

    // Total phase = 10 * 0.1 = 1.0
    // W = 2 cos(1) = 2 * 0.54 = 1.08
    let holonomy = WilsonLoopReceipt::compute_holonomy(&history, &gauge);
    assert!((holonomy - 1.08).abs() < 0.1);
    
    let receipt = WilsonLoopReceipt {
        path_hash: "test".to_string(),
        holonomy_trace: holonomy,
        curvature_sum: 0.0,
        constraint_residual: 0.0,
        bianchi_residual: 0.0,
        ym_energy: 0.0,
    };
    
    assert!(!receipt.is_admissible(0.1)); // 1.08 is far from 2.0
    assert!(receipt.is_admissible(1.5)); // 1.08 is within 1.5 of 2.0
}
