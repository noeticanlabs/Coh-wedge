use std::env;
use std::path::PathBuf;
use coh_genesis::mathlib_advisor::{MathlibLakeQuery, generate_report};
use coh_genesis::phaseloom_lite::{PhaseLoomConfig, PhaseLoomState};
use coh_genesis::npe::BoundaryReceiptSummary;
use coh_genesis::npe::{NpeProposalGraph, NpeProposal, ProposalStatus, LeanClosureStatus};

fn main() {
    println!("NPE PhaseLoom Loop: Law of Coherence [Milestone Audit Run]");
    println!("===========================================================");

    // 1. Setup paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap();
    let project_path = root_dir.join("coh-t-stack");
    
    let query = MathlibLakeQuery::new(project_path.clone());
    if !query.available {
        println!("Lake not available.");
        return;
    }

    let config = PhaseLoomConfig::default();
    let mut state = PhaseLoomState::new(&config);
    
    // Initialize Structural Memory (Lineage Graph)
    let mut graph = NpeProposalGraph::new();
    let mut root_id: Option<String> = None;

    // Run 3 iterative sweeps
    for sweep in 1..=3 {
        println!("\n--- [SWEEP {}] ---", sweep);
        
        // Alternate between Composition and Identity templates
        let target = if sweep % 2 == 1 {
            "obj.V x3 + (obj.Spend R1 + obj.Spend R2) ≤ obj.V x1 + (obj.Defect R1 + obj.Defect R2) + (obj.Authority R1 + obj.Authority R2)"
        } else {
            "obj.V x + 0 ≤ obj.V x + 0 + 0"
        };
        
        let report = generate_report(target);
        
        if let Some(template) = report.coh_template {
            println!("Pattern Detected: {:?}", template);

            let template_proof = match template {
                coh_genesis::npe::templates::CohTemplateKind::CertifiedComposition => 
                    "  unfold CohAdmissible at h1 h2\n  obtain ⟨_, h1_ineq⟩ := h1\n  obtain ⟨_, h2_ineq⟩ := h2\n  exact Coh.coh_compose_linear h1_ineq h2_ineq",
                coh_genesis::npe::templates::CohTemplateKind::IdentityCertification => 
                    "  simp",
                _ => "  sorry",
            };

            let proposal_id = format!("coh-sweep-{}", sweep);
            let mut proposal = NpeProposal {
                id: proposal_id.clone(),
                content: template_proof.to_string(),
                seed: sweep as u64,
                score: 1.0,
                content_hash: format!("hash-{}", sweep),
                depth: sweep as u32,
                parent_id: root_id.clone(),
                tau: 0,
                provenance: "EXT".to_string(),
                status: ProposalStatus::Generated,
            };

            if root_id.is_none() { root_id = Some(proposal_id.clone()); }

            // Verify with Lean
            let temp_file = project_path.join(format!("_sweep_verify_{}.lean", sweep));
            let theorem_name = if sweep % 2 == 1 { "coherence_composition" } else { "coherence_identity" };
            let lean_code = if sweep % 2 == 1 {
                format!(
                    "import Coh.Templates\nimport Coh.Boundary.LawOfCoherence\nopen Coh.Boundary\n\
                     theorem coherence_composition_sweep_{} {{X Q S : Type}} [LinearOrderedAddCommGroup S] \n\
                     (obj : CoherenceObject X Q S) (x1 x2 x3 : X) (R1 R2 : Q)\n\
                     (h1 : CohAdmissible obj x1 R1 x2)\n\
                     (h2 : CohAdmissible obj x2 R2 x3) :\n\
                     obj.V x3 + (obj.Spend R1 + obj.Spend R2) ≤ obj.V x1 + (obj.Defect R1 + obj.Defect R2) + (obj.Authority R1 + obj.Authority R2) := by\n\
                     {}\n",
                    sweep, proposal.content
                )
            } else {
                format!(
                    "import Coh.Templates\nimport Coh.Boundary.LawOfCoherence\nopen Coh.Boundary\n\
                     theorem coherence_identity_sweep_{} {{X Q S : Type}} [LinearOrderedAddCommGroup S] \n\
                     (obj : CoherenceObject X Q S) (x : X) :\n\
                     obj.V x + 0 ≤ obj.V x + 0 + 0 := by\n\
                     {}\n",
                    sweep, proposal.content
                )
            };
            
            std::fs::write(&temp_file, lean_code).unwrap();

            let output = std::process::Command::new(&query.lake_cmd)
                .args(["env", "lean", format!("_sweep_verify_{}.lean", sweep).as_str()])
                .current_dir(&project_path)
                .output()
                .unwrap();
            
            let _ = std::fs::remove_file(&temp_file);
            let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            
            // Sweep 3 injection: Simulate high Tension (Algebraic Tension)
            let tension = if sweep == 3 { 80 } else { 0 };

            if output.status.success() && !combined.contains("sorry") {
                proposal.status = ProposalStatus::Accepted;
                println!("Outcome: PROVED (ClosedNoSorry)");
                
                let receipt = BoundaryReceiptSummary {
                    domain: "lean_proof".to_string(),
                    target: target.to_string(),
                    strategy_class: "TemplateGuided".to_string(),
                    coh_template: Some(template),
                    accepted: true,
                    closure_status: LeanClosureStatus::ClosedNoSorry,
                    novelty: 1.0 / (sweep as f64),
                    tension_score: tension, 
                    sorry_detected: false,
                    outcome: "accepted".to_string(),
                    provenance: "EXT".to_string(),
                    ..BoundaryReceiptSummary::default()
                };
                state.ingest(&receipt, &config);
            } else {
                proposal.status = ProposalStatus::Rejected(combined);
                println!("Outcome: FAILED");
            }

            // Track in lineage graph with correct final status
            let parent_id = proposal.parent_id.clone();
            graph.add_proposal(proposal, parent_id);
        }
    }

    println!("\nFinal PhaseLoom Milestone Summary");
    println!("-------------------------------");
    println!("  Sweeps: 3");
    println!("  Closed Proof Receipts: {}", state.closed_proofs);
    println!("  Near Miss Receipts: {}", state.near_misses);
    println!("  BuildPassedWithSorry: {}", state.build_passed_with_sorry);
    
    println!("\nTemplate Memory");
    for (template, weight) in state.template_weights.0.iter() {
        let stats = state.template_stats.get(template).cloned().unwrap_or((0, 0));
        println!("  {}:", template);
        println!("    raw_weight: {:.1}", weight);
        println!("    successes: {}", stats.0);
        println!("    failures: {}", stats.1);
    }

    println!("\nLineage Graph");
    println!("  Nodes: {}, Edges: {}", graph.node_count(), graph.edge_count());
    println!("  Accepted Proposals: {}", graph.accepted_proposals().len());

    println!("\nStress");
    println!("  Max Tension: {}", state.max_tension);
    println!("  Intrinsic Time (τ): {:.2}", state.tau_f);
    println!("  Dilation Events: {}", state.dilation_events);

    println!("\nMilestone Audit Complete. Structural memory is warm and reinforced.");
}
