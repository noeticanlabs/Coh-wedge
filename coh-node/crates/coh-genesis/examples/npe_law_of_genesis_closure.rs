use coh_genesis::lean_proof::ProofCandidate;
use coh_genesis::phaseloom_lite::{BoundaryReceiptSummary, PhaseLoomConfig, PhaseLoomState};
use coh_npe::tools::mathlib_advisor::{
    generate_report as generate_failure_report, MathlibLakeQuery,
};
use std::env;
use std::path::PathBuf;

fn main() {
    println!("NPE Law of Genesis Production Loop");
    println!("==================================");

    // 1. Setup paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let project_path = root_dir.join("coh-t-stack");

    let mut query = MathlibLakeQuery::new(project_path.clone());
    if !query.available {
        println!("Lake not available.");
        return;
    }

    let config = PhaseLoomConfig::default();
    let mut state = PhaseLoomState::new(&config);

    // --- Strategy: MonotoneAdd ---
    println!("\n[Sweep 1] Targeting 'genesis_composition' with MonotoneAdd strategy...");

    // Search for required lemmas
    let search_query = "(_ + _ ≤ _ + _)";
    println!("Searching Mathlib for pattern '{}'...", search_query);
    let results = query.search_lemmas(search_query);

    let add_le_add = results
        .iter()
        .find(|m| m.name == "add_le_add")
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "add_le_add".to_string());

    println!("Using lemma: {}", add_le_add);

    // Final proof
    let full_proof = "  unfold GenesisAdmissible at h1 h2\n  \
                       obtain ⟨_, h1_ineq⟩ := h1\n  \
                       obtain ⟨_, h2_ineq⟩ := h2\n  \
                       rw [add_comm (obj.C p1), ← add_assoc]\n  \
                       refine le_trans (add_le_add_right h2_ineq _) ?_\n  \
                       rw [add_assoc, add_comm (obj.D p2), ← add_assoc, ← add_assoc]\n  \
                       exact add_le_add_right h1_ineq _";

    let candidate = ProofCandidate {
        id: "genesis-comp-1".to_string(),
        wildness: 0.5,
        target_theorem: "genesis_composition".to_string(),
        proof_text: full_proof.to_string(),
        proof_tactics: vec![
            "unfold".into(),
            "obtain".into(),
            "rw".into(),
            "refine".into(),
        ],
        tactic_count: 7,
        helper_lemmas: 0,
        imports: vec!["Mathlib.Algebra.Order.Monoid.Defs".into()],
        novelty: 0.9,
    };

    println!("\nVerifying candidate proof...");

    // Assemble verify file
    let temp_file = project_path.join("_genesis_verify.lean");
    let lean_code = format!(
        "import Mathlib.Algebra.Order.Monoid.Defs\nimport Coh.Boundary.LawOfGenesis\nopen Coh.Boundary\n\
         theorem genesis_composition_repro {{G P R : Type}} [OrderedAddCommMonoid R] \n\
         (obj : GenesisObject G P R) (g1 g2 g3 : G) (p1 p2 : P)\n\
         (h1 : GenesisAdmissible obj g1 p1 g2)\n\
         (h2 : GenesisAdmissible obj g2 p2 g3) :\n\
         obj.M g3 + (obj.C p1 + obj.C p2) ≤ obj.M g1 + (obj.D p1 + obj.D p2) := by\n\
         {}\n",
        candidate.proof_text
    );
    std::fs::write(&temp_file, lean_code).unwrap();

    let output = std::process::Command::new(&query.lake_cmd)
        .args(["env", "lean", "_genesis_verify.lean"])
        .current_dir(&project_path)
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&temp_file);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if output.status.success() && !combined.contains("sorry") {
        println!("SUCCESS: Law of Genesis Composition fully produced and verified!");

        let receipt = BoundaryReceiptSummary {
            domain: "LawOfGenesis".to_string(),
            target: candidate.target_theorem.clone(),
            strategy_class: "MonotoneAdd".to_string(),
            accepted: true,
            novelty: candidate.novelty,
            ..Default::default()
        };
        state.ingest(&receipt, &config);
    } else {
        println!("FAILED: Proof does not close or contains sorry.");
        if let Some(report) = generate_failure_report(
            candidate.id.as_str(),
            candidate.target_theorem.as_str(),
            &combined,
        ) {
            println!("Failure Report: {:?}", report.kind);
            let receipt = BoundaryReceiptSummary {
                domain: "LawOfGenesis".to_string(),
                target: candidate.target_theorem.clone(),
                strategy_class: "MonotoneAdd".to_string(),
                accepted: false,
                novelty: candidate.novelty,
                failure_report: Some(report),
                ..Default::default()
            };
            state.ingest(&receipt, &config);
        }
    }

    println!("\nFinal PhaseLoom Weights: {:?}", state.all_weights());
}
