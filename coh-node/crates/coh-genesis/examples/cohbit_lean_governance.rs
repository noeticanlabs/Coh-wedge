use std::process::Command;
use std::fs;
use std::path::Path;
use coh_genesis::*;
use coh_core::cohbit::{QuantumCohBit, CohBitState};
use coh_core::types::{Decision, FormalStatus};
use coh_npe::receipt::BoundaryReceiptSummary;
use coh_npe::closure::LeanClosureStatus;
use num_rational::Rational64;

fn main() {
    println!("CohBit Lean Governance Loop");
    println!("===========================");
    println!();

    // 1. Initialize GMI Governor
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
        None,
    );

    let project_path = Path::new("c:/Users/truea/OneDrive/Desktop/Coh-wedge/coh-t-stack");
    let lake_path = "c:/Users/truea/.elan/bin/lake.exe";

    // 2. Define Quantum CohBit Proposal
    let qbit = QuantumCohBit {
        amplitude_alpha: 0.6,
        amplitude_beta: 0.8,
        branch_id: None,
    };

    let branches = vec![0, 1];

    for branch in branches {
        let proposal_id = format!("cohbit_step_{}", branch);
        println!("Attempting CohBit Transition: {} (Born Weight: {})", proposal_id, qbit.born_probability(branch));

        // 3. Governor Admissibility Step
        let (admissible, trace) = governor.step(
            &proposal_id, 
            "Measurement Transition", 
            Rational64::new(1, 1), 
            Rational64::new(1, 1), 
            Rational64::new(1, 1), 
            FormalStatus::ProofCertified
        );

        if admissible {
            println!("  Governor APPROVED. Generating Lean formalization...");

            // 4. Create Lean file to formally verify the measurement law for this branch
            let lean_content = format!(
                "import Coh.Boundary.CohBit\n\
                 import Coh.Boundary.MeasurementLaw\n\n\
                 namespace Coh.Boundary\n\n\
                 theorem {}_valid : simple_born_weight ⟨(1 : ENNRat), (0 : ENNRat), (by simp)⟩ {} > 0 := by\n\
                   decide\n\n\
                 end Coh.Boundary",
                proposal_id,
                if branch == 0 { "true" } else { "false" }
            );

            let file_name = format!("{}.lean", proposal_id);
            let file_path = project_path.join("Coh").join("Boundary").join(&file_name);
            fs::write(&file_path, lean_content).unwrap();

            println!("  Executing Lean build: lake build Coh.Boundary.{}", proposal_id);
            
            let output = Command::new(lake_path)
                .args(["build", &format!("Coh.Boundary.{}", proposal_id)])
                .current_dir(project_path)
                .output()
                .expect("Failed to execute lake");

            let success = output.status.success();
            
            if success {
                println!("  [LEAN] Measurement Law VERIFIED.");
                
                // Ingest into PhaseLoom
                let receipt = BoundaryReceiptSummary {
                    target: proposal_id,
                    domain: "quantum_governance".to_string(),
                    accepted: true,
                    outcome: "certified_measurement".to_string(),
                    closure_status: LeanClosureStatus::ClosedNoSorry,
                    gamma: 1.0,
                    ..BoundaryReceiptSummary::default()
                };
                governor.atom.phaseloom.state.ingest(&receipt, &pl_config);
                println!("  [MEMORY] Record committed to PhaseLoom.");
            } else {
                println!("  [LEAN] Verification FAILED.");
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("  [ERROR] {}", stderr);
            }

            // Cleanup
            let _ = fs::remove_file(file_path);
        } else {
            println!("  Governor REJECTED: {:?}", trace.outcome);
        }
        println!();
    }

    println!("CohBit Lean Governance Loop Completed.");
}
