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
    
    // 2. Compute Full Coherence Current
    let current = CoherenceCurrent::compute(&psi);
    println!("Coherence Current J_C^mu: [J0: {:.4}, J1: {:.4}, J2: {:.4}, J3: {:.4}]", 
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

    println!("GMI Atom initialized with Spinor carrier (Spin-Coh Atom v0.2).");

    // 4. Matrix Projector Gate Demo
    let projector = coh_physics::measurement::SpinorProjector::coordinate(0);
    println!("Testing Projector Lawfulness: {}", projector.validate(1e-10));

    // 5. Emit a Spinor CohBit
    println!("Attempting Atomic Transition (CohBit Emission)...");
    
    // NOTE: In a real system, formal_status would be computed by a scanner.
    // For this demo, we assume the formal targets are accepted.
    let formal_status = FormalStatus::ProofCertified;

    let (success, trace) = atom.emit_cohbit(
        "spinor_step_v0_2", 
        "Spinor Measurement", 
        Rational64::new(1, 1), 
        Rational64::new(1, 1), 
        Rational64::new(1, 1), 
        formal_status
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
        println!("  Final Spinor Density: {:.4}", psi.density());
    }

    println!();
    println!("Coh Spinor Current & Atom v0.2 Demo Completed.");
}
