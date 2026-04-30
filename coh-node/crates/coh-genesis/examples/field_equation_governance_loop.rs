use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use num_rational::Rational64;

use coh_genesis::*;
// BoundaryReceiptSummary, LeanClosureStatus, etc. are available via coh_genesis re-exports

fn main() {
    println!("GMI Field Equation Repair Loop");
    println!("==============================");
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

    let mut governor = GmiGovernor {
        npe: npe_kernel,
        rv: rv_kernel,
        phaseloom: pl_kernel,
        env: EnvironmentalEnvelope {
            power_mj: None,
            thermal_headroom_c: None,
            wallclock_ms: 10000,
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
    };

    let project_path = Path::new("c:/Users/truea/OneDrive/Desktop/Coh-wedge/coh-t-stack");
    let lake_path = "c:/Users/truea/.elan/bin/lake.exe";
    let target_file = project_path.join("Coh/Boundary/FieldEquation.lean");

    // Define the repair proposal
    let proposal_id = "field_eq_repair_v1";
    let repaired_lemma = "theorem sub_eq_of_add_eq {a b c : ENNRat} (h1 : a = b + c) : a - c = b := by rw [h1, add_tsub_cancel_right]";
    
    // Causal parameters for the proposal
    let dist = Rational64::new(1, 4); // Very close, highly probable (Timelike)
    let c_g = Rational64::new(1, 1);
    let dt_g = Rational64::new(1, 1);

    println!("Target: {:?}", target_file);
    println!("Proposal: Fix broken lemma 'sub_eq_of_add_eq'");

    // 1. Execute GMI Governor Step
    let (admissible, trace) = governor.step(proposal_id, repaired_lemma, dist, c_g, dt_g);
    
    for event in &trace.events {
        println!("  [EVENT] {}", event);
    }

    if admissible {
        println!("  Governor APPROVED. Applying repair...");
        
        // 2. Read original file
        let original_content = fs::read_to_string(&target_file).expect("Failed to read FieldEquation.lean");
        let backup_path = target_file.with_extension("lean.bak");
        fs::write(&backup_path, &original_content).unwrap();

        // 3. Patch the file (Replacing line 51)
        let lines: Vec<&str> = original_content.lines().collect();
        let mut new_lines = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i == 50 { // Line 51 (0-indexed 50)
                new_lines.push(repaired_lemma);
            } else {
                new_lines.push(line);
            }
        }
        let new_content = new_lines.join("\n");
        fs::write(&target_file, &new_content).unwrap();

        // 4. Real Lean Verification
        println!("  Executing Lean build...");
        let output = Command::new(lake_path)
            .args(["build", "Coh.Boundary.FieldEquation"])
            .current_dir(project_path)
            .output()
            .expect("Failed to execute lake");

        let success = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if success {
            println!("  [LEAN] Build PASSED. Lemma repaired successfully.");
        } else {
            println!("  [LEAN] Build FAILED. Repair attempt was insufficient.");
            println!("  [ERROR] {}", stderr.lines().next().unwrap_or("Unknown error"));
            // Restore backup if failed? User choice, but for loop we restore.
            fs::write(&target_file, original_content).unwrap();
        }

        // Cleanup backup
        let _ = fs::remove_file(backup_path);

        // Ingest actual Lean result back into PhaseLoom
        let receipt = BoundaryReceiptSummary {
            target: proposal_id.to_string(),
            domain: "field_equation".to_string(),
            accepted: success,
            outcome: if success { "accepted".to_string() } else { "rejected".to_string() },
            closure_status: if success { LeanClosureStatus::ClosedNoSorry } else { LeanClosureStatus::BuildFailed },
            gamma: 1.0, 
            ..BoundaryReceiptSummary::default()
        };
        governor.phaseloom.state.ingest(&receipt, &pl_config);
    } else {
        println!("  Governor REJECTED.");
    }

    println!("\nLoop Completed.");
}
