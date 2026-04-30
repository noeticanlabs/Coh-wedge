use coh_core::cohbit::{CohBit};
use coh_core::atom::{CohAtom};
use coh_core::types::{Hash32, Decision};
use coh_physics::field::CohField;
use coh_physics::gauge::{CohGaugeField, WilsonLoopReceipt};
use num_rational::Rational64;
use serde_json::json;

fn main() {
    let mut results = vec![];

    // Simulation 1: Coupling Sweep
    // Measure Total Field Action as a function of coupling constant g
    for g_idx in 0..20 {
        let g = g_idx as f64 * 0.05;
        let mut field = CohField::new(g);
        
        let state_x = Hash32([0; 32]);
        let bit = CohBit {
            from_state: state_x,
            to_state: Hash32([1; 32]),
            transition_id: "test".to_string(),
            valuation_pre: Rational64::new(100, 1),
            valuation_post: Rational64::new(100, 1),
            utility: 10.0,
            rv_status: Decision::Accept,
            ..Default::default()
        };

        let atom = CohAtom {
            state_hash: state_x,
            valuation: Rational64::new(100, 1),
            admissible_bits: vec![bit.clone()],
            ..Default::default()
        };

        field.atoms.push(atom.clone());
        field.atoms.push(atom.clone());

        let interaction_cost = field.interaction_cost(&field.atoms[0], &bit);
        let total_action = field.atoms[0].compute_action(&bit, 1.0, 0.0) + interaction_cost;

        results.push(json!({
            "type": "coupling_sweep",
            "g": g,
            "interaction_cost": interaction_cost,
            "total_action": total_action
        }));
    }

    // Simulation 2: Holonomy Decay
    // Measure Wilson Loop Trace as a function of loop steps
    let mut gauge = CohGaugeField::new(3);
    gauge.connection[0][0] = 0.05; // Small rotation per step
    
    for steps in 1..50 {
        let bit = CohBit {
            from_state: Hash32([0; 32]),
            to_state: Hash32([1; 32]),
            rv_status: Decision::Accept,
            ..Default::default()
        };
        let history = coh_core::trajectory::path_integral::CohHistory {
            steps: vec![bit; steps],
        };
        let holonomy = WilsonLoopReceipt::compute_holonomy(&history, &gauge);
        
        results.push(json!({
            "type": "holonomy_decay",
            "steps": steps,
            "holonomy": holonomy
        }));
    }

    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
