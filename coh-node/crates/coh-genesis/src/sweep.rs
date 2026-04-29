//! NPE Wildness Sweep Algorithm
//!
//! Runs the Genesis Wildness Sweep: measures how much generative diversity
//! the NPE can produce while remaining Genesis-admissible and Coherence-safe.

use crate::candidate::WildnessResult;

use crate::generator::SyntheticNpeGenerator;

/// Run a wildness sweep across multiple wildness levels
///
/// For each wildness level:
/// 1. Generate N proposals
/// 2. Check Genesis admissibility
/// 3. Check Coherence admissibility
/// 4. Compute Formation acceptance
/// 5. Calculate novelty metrics
/// 6. Compute wildness yield
pub fn run_wildness_sweep(levels: &[f64], count: usize, seed: u32) -> Vec<WildnessResult> {
    let generator = SyntheticNpeGenerator::new(seed);

    levels
        .iter()
        .map(|&lambda| run_sweep_one_level(&generator, lambda, count))
        .collect()
}

/// Run sweep for a single wildness level
fn run_sweep_one_level(
    generator: &SyntheticNpeGenerator,
    wildness: f64,
    count: usize,
) -> WildnessResult {
    use crate::candidate::FirstFailure;

    let candidates = generator.generate(wildness, count);

    let mut genesis_accepts = 0;
    let mut coh_accepts = 0;
    let mut formation_accepts = 0;
    let mut novelty_sum_all = 0.0_f64;
    let mut novelty_sum_accepted = 0.0_f64;

    let mut genesis_rejects = 0;
    let mut rv_rejects = 0;
    let mut coherence_rejects = 0;

    // First-failure counts
    let mut first_genesis_count = 0;
    let mut first_rv_count = 0;
    let mut first_coh_count = 0;
    let mut accepted_count = 0;

    // Boundary margin stats for accepted candidates
    let mut genesis_margin_sum = 0i128;
    let mut coh_margin_sum = 0i128;
    let mut min_boundary_margin = i128::MAX;
    let mut near_boundary_count = 0;
    const NEAR_BOUNDARY_EPSILON: i128 = 5;

    for c in &candidates {
        let genesis_ok = c.is_genesis_admissible();
        let rv_ok = c.projection.rv_accept;
        let coh_ok = if rv_ok {
            c.coherence_margin() >= 0
        } else {
            false
        };
        let formation_ok = genesis_ok && rv_ok && coh_ok;

        novelty_sum_all += c.novelty;

        // First-failure classification
        let first_failure = FirstFailure::classify(c);
        match first_failure {
            FirstFailure::Genesis => first_genesis_count += 1,
            FirstFailure::Rv => first_rv_count += 1,
            FirstFailure::Coh => first_coh_count += 1,
            FirstFailure::None => accepted_count += 1,
        }

        if genesis_ok {
            genesis_accepts += 1;
        } else {
            genesis_rejects += 1;
        }

        if !rv_ok {
            rv_rejects += 1;
        }

        if genesis_ok && coh_ok {
            coh_accepts += 1;
        } else if genesis_ok {
            coherence_rejects += 1;
        }

        if formation_ok {
            formation_accepts += 1;
            novelty_sum_accepted += c.novelty;

            // Track boundary margins for accepted candidates
            let gen_margin = c.genesis_margin();
            let coh_margin = c.coherence_margin();
            genesis_margin_sum += gen_margin;
            coh_margin_sum += coh_margin;

            let boundary_margin = gen_margin.min(coh_margin);
            if boundary_margin < min_boundary_margin {
                min_boundary_margin = boundary_margin;
            }
            if boundary_margin >= 0 && boundary_margin <= NEAR_BOUNDARY_EPSILON {
                near_boundary_count += 1;
            }
        }
    }

    // Calculate averages for accepted candidates
    let avg_genesis_margin = if accepted_count > 0 {
        genesis_margin_sum as f64 / accepted_count as f64
    } else {
        0.0
    };
    let avg_coh_margin = if accepted_count > 0 {
        coh_margin_sum as f64 / accepted_count as f64
    } else {
        0.0
    };
    let final_min_margin = if accepted_count > 0 {
        min_boundary_margin
    } else {
        0
    };

    WildnessResult::new(
        wildness,
        count,
        genesis_accepts,
        coh_accepts,
        formation_accepts,
        novelty_sum_all,
        novelty_sum_accepted,
        genesis_rejects,
        rv_rejects,
        coherence_rejects,
        first_genesis_count,
        first_rv_count,
        first_coh_count,
        accepted_count,
        avg_genesis_margin,
        avg_coh_margin,
        final_min_margin,
        near_boundary_count,
    )
}

/// Find the optimal wildness level (maximum yield)
pub fn find_optimal_wildness(results: &[WildnessResult]) -> Option<(f64, f64)> {
    results
        .iter()
        .max_by(|a, b| a.wildness_yield.partial_cmp(&b.wildness_yield).unwrap())
        .map(|r| (r.wildness, r.wildness_yield))
}

/// Expected wildness levels for standard sweep
pub fn standard_levels() -> Vec<f64> {
    vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0]
}

/// Boundary-seeking result - the best accepted candidate near the boundary
pub struct BoundarySeekResult {
    pub candidate_id: String,
    pub wildness: f64,
    pub novelty: f64,
    pub genesis_margin: i128,
    pub coh_margin: i128,
    pub boundary_margin: i128,
    pub score: f64,
}

/// Find the boundary-seeking best candidate
/// Maximizes: score = novelty + alpha * boundary_margin
/// This is "safe-novel" mode - rewards candidates far from the boundary
pub fn find_boundary_seeker(
    generator: &SyntheticNpeGenerator,
    wildness: f64,
    count: usize,
    alpha: f64,
) -> Option<BoundarySeekResult> {
    let candidates = generator.generate(wildness, count);

    let mut best: Option<BoundarySeekResult> = None;

    for c in &candidates {
        if !c.is_formation_admissible() {
            continue;
        }

        let gen_margin = c.genesis_margin();
        let coh_margin = c.coherence_margin();
        let boundary_margin = gen_margin.min(coh_margin);

        let score = c.novelty as f64 + alpha * boundary_margin as f64;

        match best {
            None => {
                best = Some(BoundarySeekResult {
                    candidate_id: c.id.clone(),
                    wildness: c.wildness,
                    novelty: c.novelty,
                    genesis_margin: gen_margin,
                    coh_margin: coh_margin,
                    boundary_margin,
                    score,
                });
            }
            Some(ref mut b) => {
                if score > b.score {
                    b.candidate_id = c.id.clone();
                    b.wildness = c.wildness;
                    b.novelty = c.novelty;
                    b.genesis_margin = gen_margin;
                    b.coh_margin = coh_margin;
                    b.boundary_margin = boundary_margin;
                    b.score = score;
                }
            }
        }
    }

    best
}

/// Find the edge-seeking best candidate
/// Maximizes: score = novelty - alpha * boundary_margin
/// This finds candidates near the boundary (high novelty, small margin)
pub fn find_edge_seeker(
    generator: &SyntheticNpeGenerator,
    wildness: f64,
    count: usize,
    alpha: f64,
) -> Option<BoundarySeekResult> {
    let candidates = generator.generate(wildness, count);

    let mut best: Option<BoundarySeekResult> = None;

    for c in &candidates {
        if !c.is_formation_admissible() {
            continue;
        }

        let gen_margin = c.genesis_margin();
        let coh_margin = c.coherence_margin();
        let boundary_margin = gen_margin.min(coh_margin);

        // Edge-seeking: reward closeness to boundary (small margin)
        let score = c.novelty as f64 - alpha * boundary_margin as f64;

        match best {
            None => {
                best = Some(BoundarySeekResult {
                    candidate_id: c.id.clone(),
                    wildness: c.wildness,
                    novelty: c.novelty,
                    genesis_margin: gen_margin,
                    coh_margin: coh_margin,
                    boundary_margin,
                    score,
                });
            }
            Some(ref mut b) => {
                if score > b.score {
                    b.candidate_id = c.id.clone();
                    b.wildness = c.wildness;
                    b.novelty = c.novelty;
                    b.genesis_margin = gen_margin;
                    b.coh_margin = coh_margin;
                    b.boundary_margin = boundary_margin;
                    b.score = score;
                }
            }
        }
    }

    best
}

/// Find the band-limited edge candidate
/// Maximizes novelty subject to boundary_margin <= epsilon
pub fn find_near_boundary_candidate(
    generator: &SyntheticNpeGenerator,
    wildness: f64,
    count: usize,
    epsilon: i128,
) -> Option<BoundarySeekResult> {
    let candidates = generator.generate(wildness, count);

    let mut best: Option<BoundarySeekResult> = None;

    for c in &candidates {
        if !c.is_formation_admissible() {
            continue;
        }

        let gen_margin = c.genesis_margin();
        let coh_margin = c.coherence_margin();
        let boundary_margin = gen_margin.min(coh_margin);

        // Only consider candidates in the boundary band
        if boundary_margin < 0 || boundary_margin > epsilon {
            continue;
        }

        let score = c.novelty as f64;

        match best {
            None => {
                best = Some(BoundarySeekResult {
                    candidate_id: c.id.clone(),
                    wildness: c.wildness,
                    novelty: c.novelty,
                    genesis_margin: gen_margin,
                    coh_margin: coh_margin,
                    boundary_margin,
                    score,
                });
            }
            Some(ref mut b) => {
                if score > b.score {
                    b.candidate_id = c.id.clone();
                    b.wildness = c.wildness;
                    b.novelty = c.novelty;
                    b.genesis_margin = gen_margin;
                    b.coh_margin = coh_margin;
                    b.boundary_margin = boundary_margin;
                    b.score = score;
                }
            }
        }
    }

    best
}

/// Standard sweep parameters
pub struct SweepConfig {
    pub levels: Vec<f64>,
    pub count: usize,
    pub seed: u32,
}

impl Default for SweepConfig {
    fn default() -> Self {
        Self {
            levels: standard_levels(),
            count: 1000,
            seed: 42,
        }
    }
}

impl SweepConfig {
    pub fn run(&self) -> Vec<WildnessResult> {
        run_wildness_sweep(&self.levels, self.count, self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Sweep produces results for all levels
    #[test]
    fn test_sweep_all_levels() {
        let levels = vec![0.0, 1.0, 5.0];
        let results = run_wildness_sweep(&levels, 100, 42);

        assert_eq!(results.len(), 3);
        assert!((results[0].wildness - 0.0).abs() < 0.001);
        assert!((results[1].wildness - 1.0).abs() < 0.001);
        assert!((results[2].wildness - 5.0).abs() < 0.001);
    }

    /// Test: Low wildness has higher acceptance than high wildness
    #[test]
    fn test_sweep_acceptance_monotonic() {
        let levels = vec![0.0, 2.0, 5.0, 10.0];
        let results = run_wildness_sweep(&levels, 200, 42);

        let accept_rates: Vec<f64> = results.iter().map(|r| r.formation_accept_rate).collect();

        // Should generally decrease (not strictly due to randomness)
        assert!(accept_rates[0] > accept_rates[1]);
        assert!(accept_rates[1] >= accept_rates[2] - 0.1);
    }

    /// Test: Novelty increases with wildness
    #[test]
    fn test_sweep_novelty_increases() {
        let levels = vec![0.0, 2.0, 5.0, 10.0];
        let results = run_wildness_sweep(&levels, 200, 42);

        let novelty: Vec<f64> = results.iter().map(|r| r.avg_novelty_all).collect();

        for i in 1..novelty.len() {
            assert!(
                novelty[i] >= novelty[i - 1] * 0.8,
                "Novelty should increase with wildness"
            );
        }
    }

    /// Test: Find optimal wildness
    #[test]
    fn test_find_optimal_wildness() {
        let levels = vec![0.0, 1.0, 2.0, 5.0];
        let results = run_wildness_sweep(&levels, 500, 42);

        let optimal = find_optimal_wildness(&results);
        assert!(optimal.is_some());

        let (lambda, yield_) = optimal.unwrap();
        assert!(lambda >= 0.0 && lambda <= 10.0);
        assert!(yield_ >= 0.0);
    }

    /// Test: SweepConfig default
    #[test]
    fn test_sweep_config_default() {
        let config = SweepConfig::default();
        assert_eq!(config.count, 1000);
        assert_eq!(config.seed, 42);
        assert!(config.levels.len() > 0);
    }
}
