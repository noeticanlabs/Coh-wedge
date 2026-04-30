use coh_physics::{CohSpinor, current::CoherenceCurrent, measurement::SpinorProjector};
use coh_genesis::*;
use num_complex::Complex64;
use num_rational::Rational64;
use coh_core::types::FormalStatus;

fn main() {
    println!("Coh Spinor Current & Atom Demo");
    println!("==============================");
    println!();

    // 1. Initialize Coh Spinor Carrier
    // State: |psi> = [0.8, 0.6, 0, 0] (Normalized density = 1.0)
    let psi = CohSpinor::new(
        Complex64::new(0.8, 0.0),
        Complex64::new(0.6, 0.0),
        Complex64::new(0.0, 0.0),
        Complex64::new(0.0, 0.0),
    );

    println!("Carrier Density: {}", psi.density());
    
    // 2. Compute Coherence Current
    let current = CoherenceCurrent::compute(&psi);
    println!("Coherence Current J_C^mu: [J0: {}, J1: {}, J2: {}, J3: {}]", 
        current.j0, current.j1, current.j2, current.j3);

    // 3. Initialize GMI Atom with Spinor Carrier
    let npe_kernel = NpeKernel::new(
        NpeState::new(NpeConfig::default()),
        NpeGoverningState::default(),
        NpeBudget::default(),
    );
    let rv_kernel = RvKernel::new(
        RvGoverningState::default(),
        ProtectedRvBudget::default(),
    );
    let pl_config = PhaseLoomConfig::default();
    let pl_kernel = PhaseLoomKernel::new(
        PhaseLoomState::new(&pl_config),
        PhaseLoomBudget::default(),
    );

    let mut atom = GmiAtom::new(
        npe_kernel,
        rv_kernel,
        pl_kernel,
        GlobalBudgets {
            env: EnvironmentalEnvelope {
                power_mj: None,
                thermal_headroom_c: None,
                wallclock_ms: 1000,
                hardware_available: true,
                network_allowed: false,
            },
            system: SystemReserve {
                halt_available: true,
                logging_ops: 100,
                ledger_append_ops: 100,
                recovery_ops: 10,
                scheduler_ticks: 1000,
            },
            rv: ProtectedRvBudget::default(),
            npe: NpeBudget::default(),
            phaseloom: PhaseLoomBudget::default(),
        },
        Some(psi.clone()),
    );

    println!("GMI Atom initialized with Spinor carrier.");

    // 4. Project onto Record Channels
    let projector_0 = SpinorProjector { component_index: 0 };
    let projector_1 = SpinorProjector { component_index: 1 };

    let weight_0 = projector_0.born_weight(&psi);
    let weight_1 = projector_1.born_weight(&psi);

    println!("Record Channel 0 (Component 0) Born Weight: {}", weight_0);
    println!("Record Channel 1 (Component 1) Born Weight: {}", weight_1);

    // 5. Emit a Spinor CohBit
    println!("Attempting Atomic Transition (CohBit Emission)...");
    let (success, trace) = atom.emit_cohbit(
        "spinor_step_0", 
        "Spinor Measurement", 
        Rational64::new(1, 1), 
        Rational64::new(1, 1), 
        Rational64::new(1, 1), 
        FormalStatus::ProofCertified
    );

    if success {
        println!("  [SUCCESS] CohBit emitted. Outcome: {:?}", trace.outcome);
        println!("  Trace Events:");
        for event in trace.events {
            println!("    - {}", event);
        }
    } else {
        println!("  [FAILURE] Transition rejected: {:?}", trace.outcome);
    }

    if let Some(ref psi) = atom.carrier {
        println!("  Final Spinor Density: {}", psi.density());
    }

    println!();
    println!("Coh Spinor Current & Atom Demo Completed.");
}
