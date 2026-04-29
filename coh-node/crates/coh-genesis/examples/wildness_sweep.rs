use coh_genesis::{
    find_optimal_wildness, print_boundary_margin_stats, print_first_failure_table,
    print_rejection_breakdown, print_reproducibility_info, print_results_table, print_summary,
    run_wildness_sweep,
};
use std::env;

fn main() {
    println!("NPE Wildness Boundary Test");
    println!("=========================");
    println!();

    let args: Vec<String> = env::args().collect();
    let count = 1000;
    let seed = 42;
    let levels = vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0];

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("Usage: wildness_sweep [OPTIONS]");
        println!("  -c N    Candidates per level (default: 1000)");
        println!("  -s N    Random seed (default: 42)");
        println!("  -l L   Comma-separated wildness levels");
        return;
    }

    let results = run_wildness_sweep(&levels, count, seed);

    print_reproducibility_info(seed, count, &levels);
    print_results_table(&results);
    print_rejection_breakdown(&results);
    print_first_failure_table(&results);
    print_boundary_margin_stats(&results);
    print_summary(&results);

    if let Some((lambda, yield_)) = find_optimal_wildness(&results) {
        println!("==================================================");
        println!("OPTIMAL RESULT:");
        println!("  Optimal wildness level: {:.1}", lambda);
        println!("  Maximum wildness yield: {:.2}", yield_);
        println!();
        println!("This is the sweet spot where the NPE produces");
        println!("the most creative yet admissible proposals.");
        println!("==================================================");
    }
}
