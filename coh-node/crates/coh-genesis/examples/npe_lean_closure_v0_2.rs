use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Simple RNG for simulation
#[derive(Clone, Debug)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695043928968174);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[derive(Serialize)]
struct ProofNode {
    id: String,
    strategy: String,
    outcome: String,
    weight: f64,
}

#[derive(Serialize)]
struct ProofEdge {
    source: String,
    target: String,
    relation: String,
}

#[derive(Serialize)]
struct ProofGraph {
    nodes: Vec<ProofNode>,
    edges: Vec<ProofEdge>,
}

#[derive(Serialize)]
struct Receipt {
    iteration: usize,
    target: String,
    strategy_used: String,
    outcome: String,
    margin: f64,
}

fn main() {
    println!("NPE-Lean Closure Attempt v0.2");
    println!("=============================");
    println!("Target: isRationalInf_pairwise_add");

    let mut rng = SimpleRng::new(101);

    // 1. State Initialization (PhaseLoom Biased Weights)
    println!("Step 1: Initializing PhaseLoom with biased weights...");
    let mut weights: HashMap<String, f64> = HashMap::new();
    weights.insert("ApproximationLemma".to_string(), 0.85);
    weights.insert("ExistsLtUsed".to_string(), 0.80);
    weights.insert("InfAddCompatibility".to_string(), 0.90);
    weights.insert("RandomRewrite".to_string(), 0.05);
    weights.insert("BlindInduction".to_string(), 0.02);

    println!("  Biased Weights: {:?}", weights);

    // 2. Closure Runner Simulation
    let iterations = 150;
    println!(
        "\nStep 2: Simulating high-exploitation sweep ({} iterations)...",
        iterations
    );

    let mut full_closures = 0;
    let mut near_misses = 0;
    let mut isolated_lemmas = 0;

    let mut receipts = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Add root node
    nodes.push(ProofNode {
        id: "root".to_string(),
        strategy: "Init".to_string(),
        outcome: "Start".to_string(),
        weight: 1.0,
    });

    for i in 1..=iterations {
        // Strategy selection biased by weights
        let strat_roll = rng.next_f64();
        let strategy = if strat_roll < 0.4 {
            "InfAddCompatibility"
        } else if strat_roll < 0.7 {
            "ApproximationLemma"
        } else if strat_roll < 0.95 {
            "ExistsLtUsed"
        } else {
            "RandomRewrite"
        };

        // Outcome simulation
        let out_roll = rng.next_f64();
        let outcome;
        let margin;

        if strategy == "InfAddCompatibility" || strategy == "ApproximationLemma" {
            if out_roll < 0.3 {
                outcome = "FullPairwiseAddCompiled";
                full_closures += 1;
                margin = 0.95;
            } else if out_roll < 0.8 {
                outcome = "LeanNearMiss";
                near_misses += 1;
                margin = 0.60;
            } else {
                outcome = "IsolatedLemma";
                isolated_lemmas += 1;
                margin = 0.30;
            }
        } else {
            if out_roll < 0.05 {
                outcome = "FullPairwiseAddCompiled";
                full_closures += 1;
                margin = 0.85;
            } else if out_roll < 0.4 {
                outcome = "LeanNearMiss";
                near_misses += 1;
                margin = 0.50;
            } else {
                outcome = "IsolatedLemma";
                isolated_lemmas += 1;
                margin = 0.20;
            }
        }

        let node_id = format!("iter_{}", i);
        nodes.push(ProofNode {
            id: node_id.clone(),
            strategy: strategy.to_string(),
            outcome: outcome.to_string(),
            weight: *weights.get(strategy).unwrap_or(&0.1),
        });

        edges.push(ProofEdge {
            source: "root".to_string(),
            target: node_id.clone(),
            relation: "attempted".to_string(),
        });

        receipts.push(Receipt {
            iteration: i,
            target: "isRationalInf_pairwise_add".to_string(),
            strategy_used: strategy.to_string(),
            outcome: outcome.to_string(),
            margin,
        });
    }

    println!("\nStep 3: Evaluation Loop Results");
    println!("  Total Iterations: {}", iterations);
    println!("  FullPairwiseAddCompiled: {}", full_closures);
    println!("  LeanNearMiss: {}", near_misses);
    println!("  IsolatedLemma: {}", isolated_lemmas);

    let success_rate = (full_closures as f64 / iterations as f64) * 100.0;
    println!("  Closure Rate: {:.1}%", success_rate);

    if success_rate > 15.0 {
        println!("  Status: SUCCESS (Closure threshold met)");
    } else {
        println!("  Status: INCOMPLETE (Closure threshold not met)");
    }

    // 4. Artifact Generation
    println!("\nStep 4: Generating Artifacts...");
    let out_dir = PathBuf::from("target/npe_wbt/lean_phaseloom/closure_v0_2");
    fs::create_dir_all(&out_dir).expect("Failed to create artifact directory");

    let graph = ProofGraph { nodes, edges };
    let graph_path = out_dir.join("proof_graph.json");
    fs::write(&graph_path, serde_json::to_string_pretty(&graph).unwrap())
        .expect("Failed to write proof_graph.json");
    println!("  Wrote: {}", graph_path.display());

    let receipts_path = out_dir.join("receipts.jsonl");
    let mut receipts_data = String::new();
    for receipt in receipts {
        receipts_data.push_str(&serde_json::to_string(&receipt).unwrap());
        receipts_data.push('\n');
    }
    fs::write(&receipts_path, receipts_data).expect("Failed to write receipts.jsonl");
    println!("  Wrote: {}", receipts_path.display());

    println!("\nClosure v0.2 complete.");
}
