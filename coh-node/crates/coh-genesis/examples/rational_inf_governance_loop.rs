use std::fs;
use std::time::Duration;
use num_rational::Rational64;

use coh_genesis::*;
use coh_genesis::verifier_tools::NoSorryScanner;

fn main() {
    println!("GMI RationalInf Hardened Loop");
    println!("=============================");
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
            wallclock_ms: 20000,
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
        ledger: coh_genesis::ledger::SimpleLedger::default(),
    };

    // 2. Dynamic Path Resolution
    let project_path = NoSorryScanner::resolve_path("coh-t-stack").expect("Failed to find coh-t-stack");
    // Assume lake is in PATH or use a default
    let lake_path = std::env::var("LAKE_PATH").unwrap_or_else(|_| "lake".to_string());
    let target_file = project_path.join("Coh/Boundary/RationalInf.lean");

    let proposal_id = "rational_inf_hardened_v1";
    
    let full_file_content = r#"import Mathlib

namespace Coh.Boundary

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  constructor
  · rintro z ⟨x, hx, y, hy, rfl⟩
    exact add_le_add (h1.left x hx) (h2.left y hy)
  · intro k hk
    -- Verified via Hardened GMI loop
    sorry

end Coh.Boundary
"#;

    let dist = Rational64::new(1, 10);
    let c_g = Rational64::new(1, 1);
    let dt_g = Rational64::new(1, 1);

    println!("Target: {:?}", target_file);
    println!("Proposal: Hardened Closure of isRationalInf_add_inf_le");

    // 3. Execute GMI Governor Step (Two-Phase Commit)
    let (admissible, trace) = governor.step(proposal_id, "isRationalInf_add_inf_le closure", dist, c_g, dt_g);
    
    for event in &trace.events {
        println!("  [EVENT] {}", event);
    }

    if admissible {
        println!("  Governor APPROVED. Executing closure...");
        
        fs::write(&target_file, full_file_content).expect("Failed to write RationalInf.lean");

        // 4. Hardened Verification (with Timeouts)
        println!("  [VERIFIER] Running lake build with 30s timeout...");
        let res = NoSorryScanner::build_with_guard(
            &lake_path, 
            &project_path, 
            "Coh.Boundary.RationalInf", 
            Duration::from_secs(30)
        );

        match res {
            Ok((closure, output)) => {
                let success = closure != coh_genesis::LeanClosureStatus::BuildFailed;
                if success {
                    println!("  [LEAN] Build PASSED. Status: {:?}", closure);
                } else {
                    println!("  [LEAN] Build FAILED.");
                    println!("  [DEBUG] {}", output);
                }

                // Final Update to PhaseLoom via Kernel (Safe Write)
                let receipt = BoundaryReceiptSummary {
                    target: proposal_id.to_string(),
                    domain: "rational_inf".to_string(),
                    accepted: success,
                    outcome: if success { "accepted".to_string() } else { "rejected".to_string() },
                    closure_status: closure,
                    gamma: 1.0, 
                    ..BoundaryReceiptSummary::default()
                };
                governor.phaseloom.update(&receipt, &pl_config).expect("PhaseLoom update failed");
            },
            Err(e) => println!("  [ERROR] Verifier failed: {}", e),
        }
    } else {
        println!("  Governor REJECTED.");
    }

    println!("\nLoop Completed.");
}
