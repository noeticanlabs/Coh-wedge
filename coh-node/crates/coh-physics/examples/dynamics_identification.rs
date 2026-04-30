use coh_core::cohbit::{CohBit, CohBitLaw};
use coh_core::atom::{CohAtom};
use coh_core::types::{Decision};
use serde_json::json;
use std::time::Instant;

fn main() {
    let mut dynamics_data = vec![];
    let mut benchmark_data = vec![];

    // 1. Verifier Rejection Pressure: Entropy vs Temperature (Tau)
    // Measures the 'computational friction' as the proposal engine (soft) 
    // deviates from the verifier-gated execution (exec).
    for tau_idx in 1..50 {
        let tau = tau_idx as f64 * 0.1;
        let mut bits = vec![];
        for i in 0..10 {
            bits.push(CohBit {
                utility: i as f64,
                valuation_pre: num_rational::Rational64::new(100, 1),
                valuation_post: num_rational::Rational64::new(95, 1),
                spend: num_rational::Rational64::new(10, 1), // Margin -5 (Inexecutable)
                rv_status: Decision::Accept,
                ..Default::default()
            });
        }
        // Add one executable bit to allow for a non-zero exec partition function
        bits[0].spend = num_rational::Rational64::new(0, 1); // Margin +5 (Executable)

        CohBitLaw::compute_soft_probabilities(&mut bits, tau, 1.0);
        CohBitLaw::compute_exec_probabilities(&mut bits, tau);
        
        let soft_entropy = coh_core::cohbit::CohBitThermodynamics::soft_entropy(&bits);
        let exec_entropy = coh_core::cohbit::CohBitThermodynamics::exec_entropy(&bits);
        let pressure = soft_entropy - exec_entropy;

        dynamics_data.push(json!({
            "type": "entropy_sweep",
            "tau": tau,
            "soft_entropy": soft_entropy,
            "exec_entropy": exec_entropy,
            "rejection_pressure": pressure
        }));
    }

    // 2. Rigorous Benchmarks: SU(2) Holonomy Latency (Isolated)
    for size_idx in 1..11 {
        let steps = size_idx * 1000;
        let mut gauge = coh_physics::gauge::CohGaugeField::new(3);
        gauge.connection[0][0] = 0.01;
        gauge.connection[0][1] = 0.02;
        gauge.connection[0][2] = 0.03;
        
        let bit = CohBit {
            rv_status: Decision::Accept,
            ..Default::default()
        };
        let history = coh_core::trajectory::path_integral::CohHistory {
            steps: vec![bit; steps],
        };
        
        // Measure ONLY the non-Abelian path-ordered product
        let start = Instant::now();
        let holonomy = coh_physics::gauge::WilsonLoopReceipt::compute_holonomy(&history, &gauge);
        let duration = start.elapsed().as_micros();

        benchmark_data.push(json!({
            "type": "latency_bench",
            "steps": steps,
            "latency_us": duration,
            "result_holonomy": holonomy
        }));
    }

    // 3. Rigorous Memory Footprint (Stack Size)
    let memory_metadata = json!({
        "cohbit_stack_bytes": std::mem::size_of::<CohBit>(),
        "cohatom_stack_bytes": std::mem::size_of::<CohAtom>(),
    });

    let report = json!({
        "metadata": memory_metadata,
        "dynamics": dynamics_data,
        "benchmarks": benchmark_data
    });

    // Explicitly write as UTF-8 string to stdout
    let json_output = serde_json::to_string_pretty(&report).unwrap();
    println!("{}", json_output);
}
