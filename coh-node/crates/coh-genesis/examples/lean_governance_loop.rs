use std::process::Command;
use std::fs;
use std::path::Path;
use num_rational::Rational64;

use coh_genesis::*;
// We already have BoundaryReceiptSummary, LeanClosureStatus, etc. via coh_genesis re-exports

fn main() {
    println!("GMI Lean Governance Loop");
    println!("=========================");
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
    };

    let project_path = Path::new("c:/Users/truea/OneDrive/Desktop/Coh-wedge/coh-t-stack");
    let lake_path = "c:/Users/truea/.elan/bin/lake.exe";

    // Define two proof attempts
    let attempts = vec![
        (
            "loop_step_1",
            "theorem test_1 : 1 + 1 = 2 := rfl",
            Rational64::new(1, 2), // distance = 0.5 (Timelike)
            "Timelike"
        ),
        (
            "loop_step_2",
            "theorem test_2 : 1 + 1 = 3 := rfl",
            Rational64::new(10, 1), // distance = 10 (will be Spacelike if cG=1, dt=1)
            "Spacelike"
        ),
    ];

    let c_g = Rational64::new(1, 1);
    let dt_g = Rational64::new(1, 1);

    for (id, content, dist, expected_class) in attempts {
        println!("Attempting: {} (Expected Class: {})", id, expected_class);
        
        // 1. Execute GMI Governor Step
        let (admissible, trace) = governor.step(id, content, dist, c_g, dt_g);
        
        for event in &trace.events {
            println!("  [EVENT] {}", event);
        }

        if admissible {
            println!("  Governor APPROVED. Executing Lean build...");
            
            // 2. Real Lean Verification
            let file_name = format!("{}.lean", id);
            let file_path = project_path.join("Coh").join(&file_name);
            fs::write(&file_path, format!("namespace Coh\n{}\nend Coh", content)).unwrap();
            
            let output = Command::new(lake_path)
                .args(["build", &format!("Coh.{}", id)])
                .current_dir(project_path)
                .output()
                .expect("Failed to execute lake");

            let success = output.status.success();
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if success {
                println!("  [LEAN] Build PASSED.");
            } else {
                println!("  [LEAN] Build FAILED.");
                println!("  [ERROR] {}", stderr.lines().next().unwrap_or("Unknown error"));
            }

            // Cleanup
            let _ = fs::remove_file(file_path);

            // Ingest actual Lean result back into PhaseLoom
            let receipt = BoundaryReceiptSummary {
                target: id.to_string(),
                domain: "lean".to_string(),
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
        println!();
    }

    println!("Loop Completed.");
}
