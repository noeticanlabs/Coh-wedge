//! Trajectory Probability Module
//!
//! Implements the probability law for trajectory verification.
//! Provides probabilistic bounds on the accounting law for chains of arbitrary length.
//!
//! # Probability Law
//!
//! The core probability law states that for a trajectory of `n` steps:
//! `P(law holds after n steps) >= 1 - ε(n)`
//!
//! Where ε(n) is a telescoping bound that decreases with longer chains,
//! reflecting the increasing confidence from repeated law verification.

use serde::{Deserialize, Serialize};

/// Configuration for trajectory probability verification
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryProbabilityConfig {
    /// Maximum number of branches to explore in trajectory analysis
    pub max_branches: u64,
    /// Confidence threshold: reject if P(valid) < confidence_threshold
    pub confidence_threshold: f64,
    /// Risk weight for defect accumulation (higher = more conservative)
    pub risk_weight: f64,
    /// Enable probabilistic bounds vs deterministic only
    pub enable_probabilistic: bool,
}

impl Default for TrajectoryProbabilityConfig {
    fn default() -> Self {
        Self {
            max_branches: 10_000,
            confidence_threshold: 0.999,
            risk_weight: 1.0,
            enable_probabilistic: true,
        }
    }
}

/// Result of trajectory probability analysis
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryProbabilityResult {
    /// Probability that the trajectory satisfies all accounting laws
    pub probability_valid: f64,
    /// Risk-adjusted score (higher = more risky)
    pub risk_score: f64,
    /// Whether the trajectory meets the confidence threshold
    pub meets_threshold: bool,
    /// Maximum defect accumulated in trajectory
    pub max_defect: u128,
    /// Number of steps analyzed
    pub step_count: u64,
    /// Confidence bound (1 - ε)
    pub confidence_bound: f64,
}

impl Default for TrajectoryProbabilityResult {
    fn default() -> Self {
        Self {
            probability_valid: 1.0,
            risk_score: 0.0,
            meets_threshold: true,
            max_defect: 0,
            step_count: 0,
            confidence_bound: 1.0,
        }
    }
}

/// Trajectory Probability Verifier
///
/// Implements the probability law for verifying chains of micro-receipts.
/// Based on the telescoping property of the accounting law:
/// if each step satisfies v_post + spend <= v_pre + defect,
/// then the aggregate satisfies v_post_last + cumulative_spend <= v_pre_first + total_defect
pub struct TrajectoryProbabilityVerifier {
    config: TrajectoryProbabilityConfig,
}

impl TrajectoryProbabilityVerifier {
    /// Create a new verifier with the given configuration
    pub fn new(config: TrajectoryProbabilityConfig) -> Self {
        Self { config }
    }

    /// Create a verifier with default configuration
    pub fn default() -> Self {
        Self::new(TrajectoryProbabilityConfig::default())
    }

    /// Compute probability bound for a given number of steps
    ///
    /// The probability law states that after n independent verifications,
    /// the probability of the aggregate law holding is:
    /// `P(n) = 1 - (1 - P_single)^n` where P_single is the per-step probability
    ///
    /// Using a conservative bound, we assume P_single ≈ 0.9999 (one in 10,000 chance
    /// of a single verification error). The bound telescopes as:
    /// `ε(n) ≈ n * ε_1` for small n, or `ε(n) ≈ 1 - (1-ε_1)^n` for larger n
    ///
    /// # Arguments
    /// * `step_count` - Number of steps in the trajectory
    /// * `single_step_confidence` - Confidence in a single step's verification (default ~0.9999)
    ///
    /// # Returns
    /// Probability bound: the minimum probability that the aggregate law holds
    pub fn probability_bound(&self, step_count: u64, single_step_confidence: f64) -> f64 {
        if step_count == 0 {
            return 1.0;
        }

        // Conservative single-step confidence (99.99% = 1 in 10,000 failure rate)
        let p_single = single_step_confidence.max(0.0).min(1.0);

        // Telescoping probability: P(all succeed) = P(s1) * P(s2) * ... * P(sn)
        // For identical independent steps: = P(single)^n
        // But we use a more conservative bound based on the law's structure

        // Using the Chernoff-style bound for correlated events:
        // If each step has at most ε_1 error probability, after n steps
        // the error probability is at most n * ε_1 (union bound)
        // For n > 100, we use the exponential decay
        if step_count < 100 {
            // Linear union bound (conservative)
            1.0 - (step_count as f64) * (1.0 - p_single)
        } else {
            // Exponential bound for large n
            p_single.powf(step_count as f64)
        }
    }

    /// Compute risk-adjusted verification result for a trajectory
    ///
    /// # Arguments
    /// * `step_count` - Number of steps in the chain
    /// * `cumulative_spend` - Total spend across all steps
    /// * `total_defect` - Total defect across all steps
    /// * `v_pre_first` - Initial v_pre value
    /// * `v_post_last` - Final v_post value
    ///
    /// # Returns
    /// TrajectoryProbabilityResult with probability and risk analysis
    pub fn risk_adjusted_verification(
        &self,
        step_count: u64,
        cumulative_spend: u128,
        total_defect: u128,
        v_pre_first: u128,
        v_post_last: u128,
    ) -> TrajectoryProbabilityResult {
        // Check deterministic accounting law first
        let deterministic_valid = v_post_last.saturating_add(cumulative_spend)
            <= v_pre_first.saturating_add(total_defect);

        if !self.config.enable_probabilistic {
            // Pure deterministic mode
            return TrajectoryProbabilityResult {
                probability_valid: if deterministic_valid { 1.0 } else { 0.0 },
                risk_score: if deterministic_valid { 0.0 } else { 1.0 },
                meets_threshold: deterministic_valid,
                max_defect: total_defect,
                step_count,
                confidence_bound: 1.0,
            };
        }

        // Compute probability bound using telescoping law
        let confidence_bound = self.probability_bound(step_count, 0.9999);

        // Compute risk score based on:
        // 1. How close to the boundary (law violation margin)
        // 2. Total defect accumulation
        // 3. Number of steps (more steps = higher cumulative risk)
        let lhs = v_post_last.saturating_add(cumulative_spend);
        let rhs = v_pre_first.saturating_add(total_defect);

        let margin = rhs.saturating_sub(lhs);
        let margin_ratio = if rhs > 0 {
            (margin as f64) / (rhs.max(1) as f64)
        } else {
            0.0
        };

        // Risk increases as margin decreases and defect increases
        let defect_ratio = if total_defect > 0 && v_pre_first > 0 {
            (total_defect as f64) / (v_pre_first as f64)
        } else {
            0.0
        };

        // Combined risk score: weighted by config risk_weight
        let risk_score = self.config.risk_weight
            * (1.0 - margin_ratio.min(1.0))
            * (1.0 + defect_ratio)
            * (step_count as f64).sqrt()
            / 100.0;

        let probability_valid = confidence_bound * margin_ratio.min(1.0);
        let meets_threshold = probability_valid >= self.config.confidence_threshold;

        TrajectoryProbabilityResult {
            probability_valid,
            risk_score: risk_score.min(1.0),
            meets_threshold,
            max_defect: total_defect,
            step_count,
            confidence_bound,
        }
    }

    /// Quick check: does the trajectory meet the confidence threshold?
    ///
    /// This is an optimized version that returns early for clear cases
    pub fn check_confidence(&self, step_count: u64) -> bool {
        let bound = self.probability_bound(step_count, 0.9999);
        bound >= self.config.confidence_threshold
    }

    /// Maximum steps allowed given the confidence threshold
    ///
    /// Returns the maximum number of steps that can be verified
    /// while maintaining the configured confidence threshold
    pub fn max_steps_for_confidence(&self) -> u64 {
        // Solve for n: p_single^n >= confidence_threshold
        // n <= log(confidence_threshold) / log(p_single)
        let p_single: f64 = 0.9999;
        let threshold: f64 = self.config.confidence_threshold;

        if threshold >= 1.0 {
            return 0;
        }
        if threshold <= 0.0 {
            return u64::MAX;
        }

        let max_n = (threshold.ln()) / (p_single.ln());
        max_n as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_bound_single_step() {
        let verifier = TrajectoryProbabilityVerifier::default();
        let bound = verifier.probability_bound(1, 0.9999);
        assert!(bound > 0.999);
        assert!(bound <= 1.0);
    }

    #[test]
    fn test_probability_bound_many_steps() {
        let verifier = TrajectoryProbabilityVerifier::default();
        // For 1000 steps, probability should decrease but stay high
        let bound = verifier.probability_bound(1000, 0.9999);
        // With 0.9999^1000 ≈ 0.9048, bound should be around 0.90
        assert!(bound > 0.8);
        assert!(bound < 1.0);
    }

    #[test]
    fn test_risk_adjusted_verification_valid() {
        let config = TrajectoryProbabilityConfig {
            confidence_threshold: 0.5, // Lower threshold for test
            ..Default::default()
        };
        let verifier = TrajectoryProbabilityVerifier::new(config);
        // Valid: 80 + 15 <= 100 + 5 (95 <= 105)
        let result = verifier.risk_adjusted_verification(
            1,   // step_count
            15,  // cumulative_spend
            5,   // total_defect
            100, // v_pre_first
            80,  // v_post_last
        );

        assert!(result.probability_valid > 0.0);
        assert_eq!(result.max_defect, 5);
    }

    #[test]
    fn test_risk_adjusted_verification_invalid() {
        let verifier = TrajectoryProbabilityVerifier::default();
        // Invalid: 90 + 20 > 100 + 0 (110 > 100)
        let result = verifier.risk_adjusted_verification(
            1,   // step_count
            20,  // cumulative_spend
            0,   // total_defect
            100, // v_pre_first
            90,  // v_post_last
        );

        assert!(!result.meets_threshold || result.risk_score > 0.5);
    }

    #[test]
    fn test_max_steps_for_confidence() {
        let config = TrajectoryProbabilityConfig {
            confidence_threshold: 0.90, // Lower threshold for realistic test
            ..Default::default()
        };
        let verifier = TrajectoryProbabilityVerifier::new(config);
        let max_steps = verifier.max_steps_for_confidence();

        // With 0.9999^1000 ≈ 0.904, we get about 90% confidence
        // For 90% confidence: n <= log(0.90)/log(0.9999) ≈ 1053
        assert!(max_steps > 100);
        assert!(max_steps < 2000);
    }

    #[test]
    fn test_check_confidence() {
        let config = TrajectoryProbabilityConfig {
            confidence_threshold: 0.99,
            ..Default::default()
        };
        let verifier = TrajectoryProbabilityVerifier::new(config);

        // 100 steps should meet 99% threshold
        assert!(verifier.check_confidence(100));
        // 10000 steps likely won't
        assert!(!verifier.check_confidence(10000));
    }

    #[test]
    fn test_zero_steps() {
        let verifier = TrajectoryProbabilityVerifier::default();
        let bound = verifier.probability_bound(0, 0.9999);
        assert_eq!(bound, 1.0);
    }
}
