use coh_npe::candidate::WildnessResult;
use crate::sweep::find_optimal_wildness;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn print_results_table(results: &[WildnessResult]) {
    println!();
    println!("==================================================");
    println!("NPE Wildness Boundary Test Results");
    println!("==================================================");
    println!();
    println!(
        "{:>6} | {:>10} | {:>10} | {:>12} | {:>8} | {:>8}",
        "lambda", "GenAccept", "CohAccept", "FormAccept", "Novelty", "Yield"
    );
    println!("--------------------------------------------------");
    for r in results {
        println!(
            "{:>6.1} | {:>10.2} | {:>10.2} | {:>12.2} | {:>8.1} | {:>8.2}",
            r.wildness,
            r.genesis_accept_rate,
            r.coh_accept_rate,
            r.formation_accept_rate,
            r.avg_novelty_all,
            r.wildness_yield
        );
    }
    println!("--------------------------------------------------");
    println!();
}

pub fn print_rejection_breakdown(results: &[WildnessResult]) {
    println!();
    println!("==================================================");
    println!("Rejection Breakdown by Wildness Level");
    println!("==================================================");
    println!();

    println!(
        "{:>6} | {:>14} | {:>12} | {:>14}",
        "lambda", "Genesis Reject", "RV Reject", "Coh Reject"
    );
    println!("--------------------------------------------------");
    for r in results {
        println!(
            "{:>6.1} | {:>14} | {:>12} | {:>14}",
            r.wildness, r.genesis_rejects, r.rv_rejects, r.coherence_rejects
        );
    }
    println!("--------------------------------------------------");
    println!();
}

pub fn print_summary(results: &[WildnessResult]) {
    println!("Summary:");
    println!("-------");
    if let Some((lambda, yield_)) = find_optimal_wildness(results) {
        println!("  Optimal wildness level: {:.1}", lambda);
        println!("  Maximum yield at optimal: {:.2}", yield_);
    }
    let formation_rates: Vec<f64> = results.iter().map(|r| r.formation_accept_rate).collect();
    let novs: Vec<f64> = results.iter().map(|r| r.avg_novelty_all).collect();
    let min_form = formation_rates.iter().cloned().fold(f64::MAX, f64::min);
    let max_form = formation_rates.iter().cloned().fold(f64::MIN, f64::max);
    println!(
        "  Formation acceptance range: {:.2} - {:.2}",
        min_form, max_form
    );
    let min_nov = novs.iter().cloned().fold(f64::MAX, f64::min);
    let max_nov = novs.iter().cloned().fold(f64::MIN, f64::max);
    println!("  Novelty range: {:.1} - {:.1}", min_nov, max_nov);
    println!();
}

pub fn export_csv<P: AsRef<Path>>(results: &[WildnessResult], path: P) -> io::Result<()> {
    let mut file = File::create(path.as_ref())?;

    file.write_all(b"wildness,total,genesis_accept_rate,coh_accept_rate,formation_accept_rate,avg_novelty_all,avg_novelty_accepted,wildness_yield,genesis_rejects,rv_rejects,coherence_rejects\n")?;

    for r in results {
        let line = format!(
            "{:.1},{},{:.4},{:.4},{:.4},{:.2},{:.2},{:.4},{},{},{}\n",
            r.wildness,
            r.total,
            r.genesis_accept_rate,
            r.coh_accept_rate,
            r.formation_accept_rate,
            r.avg_novelty_all,
            r.avg_novelty_accepted,
            r.wildness_yield,
            r.genesis_rejects,
            r.rv_rejects,
            r.coherence_rejects
        );
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}

pub fn export_json<P: AsRef<Path>>(results: &[WildnessResult], path: P) -> io::Result<()> {
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut file = File::create(path.as_ref())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn print_reproducibility_info(seed: u32, count: usize, levels: &[f64]) {
    println!();
    println!("==================================================");
    println!("Benchmark Configuration");
    println!("==================================================");
    println!();
    println!("  generator = SyntheticNpeGenerator");
    println!("  seed = {}", seed);
    println!("  count_per_level = {}", count);
    println!("  levels = {:?}", levels);
    println!("  version = coh-genesis");
    println!();
}

pub fn print_first_failure_table(results: &[WildnessResult]) {
    println!();
    println!("==================================================");
    println!("First-Failure Classification");
    println!("==================================================");
    println!();

    println!(
        "{:>6} | {:>14} | {:>10} | {:>10} | {:>10}",
        "lambda", "FirstGenesis", "FirstRV", "FirstCoh", "Accepted"
    );
    println!("--------------------------------------------------");
    for r in results {
        println!(
            "{:>6.1} | {:>14} | {:>10} | {:>10} | {:>10}",
            r.wildness,
            r.first_genesis_count,
            r.first_rv_count,
            r.first_coh_count,
            r.accepted_count
        );
    }
    println!("--------------------------------------------------");
    println!();
}

pub fn print_boundary_margin_stats(results: &[WildnessResult]) {
    println!();
    println!("==================================================");
    println!("Accepted Candidate Boundary Margins");
    println!("==================================================");
    println!();

    println!(
        "{:>6} | {:>16} | {:>16} | {:>18} | {:>16}",
        "lambda", "AvgGenMargin", "AvgCohMargin", "MinBoundaryMargin", "NearBoundary%"
    );
    println!("--------------------------------------------------");
    for r in results {
        let near_pct = r.near_boundary_percent;
        println!(
            "{:>6.1} | {:>16.1} | {:>16.1} | {:>18} | {:>15.1}%",
            r.wildness,
            r.avg_genesis_margin_accepted,
            r.avg_coh_margin_accepted,
            r.min_boundary_margin_accepted,
            near_pct
        );
    }
    println!("--------------------------------------------------");
    println!();
}

use crate::sweep::BoundarySeekResult;

pub fn print_boundary_seeker_result(result: &BoundarySeekResult) {
    println!();
    println!("==================================================");
    println!("Boundary-Seeking Result");
    println!("==================================================");
    println!();
    println!("  candidate_id: {}", result.candidate_id);
    println!("  wildness: {:.1}", result.wildness);
    println!("  novelty: {:.1}", result.novelty);
    println!("  delta_gen: {}", result.genesis_margin);
    println!("  delta_coh: {}", result.coh_margin);
    println!("  boundary_margin: {}", result.boundary_margin);
    println!("  score: {:.2}", result.score);
    println!();
}
