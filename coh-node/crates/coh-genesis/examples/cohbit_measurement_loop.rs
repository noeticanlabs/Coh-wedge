use coh_genesis::*;
use coh_core::cohbit::{QuantumCohBit, CohBitState};
use coh_core::types::{Decision, FormalStatus};
use coh_npe::receipt::BoundaryReceiptSummary;
use coh_npe::closure::LeanClosureStatus;
use num_rational::Rational64;

fn main() {
    println!("CohBit Measurement Loop (Quantum/GMI Integration)");
    println!("===============================================");
    println!();

    // 1. Initialize GMI Atom / Governor
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

    let mut governor = GmiGovernor::new(
        npe_kernel,
        rv_kernel,
        pl_kernel,
        EnvironmentalEnvelope {
            power_mj: None,
            thermal_headroom_c: None,
            wallclock_ms: 1000,
            hardware_available: true,
            network_allowed: false,
        },
        SystemReserve {
            halt_available: true,
            logging_ops: 100,
            ledger_append_ops: 100,
            recovery_ops: 10,
            scheduler_ticks: 1000,
        },
    );

    // 2. Define a Quantum CohBit (Superposed state)
    // |psi> = 0.6|0> + 0.8|1>  (Normalised: 0.36 + 0.64 = 1.0)
    let mut qbit = QuantumCohBit {
        amplitude_alpha: 0.6,
        amplitude_beta: 0.8,
        branch_id: None,
    };

    println!("Initial CohBit state: Superposed");
    println!("Amplitudes: α=0.6, β=0.8");
    println!("Born Probabilities: P(0)=0.36, P(1)=0.64");
    println!();

    // 3. Execution Loop: Measurement as Admissible Record Formation
    let branches = vec![0, 1];
    
    for branch in branches {
        println!("--- CohBit Measurement Attempt: Branch {} ---", branch);
        
        // 3a. Projection
        let prob = qbit.born_probability(branch);
        println!("  [PROJECTION] Branch weight (Born probability): {}", prob);

        // 3b. RV Verification (Decision)
        // A branch is admissible only if its weight is non-zero (simplified verifier law)
        let decision = qbit.measure(branch);
        println!("  [VERIFIER] Admissibility check: {:?}", decision);

        if decision == Decision::Accept {
            println!("  [COHBIT] Transitioning: Superposed -> CandidateRecord -> RVAccepted");
            
            // 3c. Governor Step (Formalising the record)
            let proposal_id = format!("cohbit_branch_{}", branch);
            let content = format!("Quantum Measurement Branch {}", branch);
            let dist = Rational64::new(1, 1);
            let c_g = Rational64::new(1, 1);
            let dt_g = Rational64::new(1, 1);
            
            // In a CohBit measurement, the "record" becomes actual through governor certification.
            let (admissible, mut trace) = governor.step(
                &proposal_id, 
                &content, 
                dist, 
                c_g, 
                dt_g, 
                FormalStatus::ProofCertified
            );
            trace.cohbit_state = CohBitState::RVAccepted;

            if admissible {
                println!("  [RECEIPT] Ledger Entry Created: {}", trace.step_id);
                println!("  [CONTINUATION] Lüders state-reduction occurred.");
                
                // Ingest into PhaseLoom (Updating Memory Law)
                let receipt = BoundaryReceiptSummary {
                    target: proposal_id,
                    domain: "quantum_measurement".to_string(),
                    accepted: true,
                    outcome: "born_weighted_accepted".to_string(),
                    closure_status: LeanClosureStatus::ClosedNoSorry,
                    gamma: 1.0,
                    provenance: "DER".to_string(),
                    ..BoundaryReceiptSummary::default()
                };
                governor.phaseloom.state.ingest(&receipt, &pl_config);
                
                println!("  [MEMORY] PhaseLoom updated with branch outcome (Transition Committed).");
            } else {
                println!("  [GOVERNOR] Global admissibility check FAILED (Inadmissible transition).");
            }
        } else {
            println!("  [REJECT] Zero probability branch or verifier block.");
        }
        println!();
    }

    println!("CohBit Measurement Loop Completed.");
}
