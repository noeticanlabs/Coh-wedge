use crate::trajectory::domain::COH_PRECISION;
use crate::trajectory::types::AdmissibleTrajectory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
pub struct PathEvaluation {
    pub safety_bottleneck: u128,
    pub progress: u128,
    pub normalized_cost: u128,
}

impl PartialOrd for PathEvaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathEvaluation {
    /// Lexicographic comparison: Safety > Progress > -Cost (All u128 fixed-point)
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 1. Safety Bottleneck (Min-Margin)
        let s_cmp = self.safety_bottleneck.cmp(&other.safety_bottleneck);
        if s_cmp != std::cmp::Ordering::Equal {
            return s_cmp;
        }

        // 2. Progress Index
        let p_cmp = self.progress.cmp(&other.progress);
        if p_cmp != std::cmp::Ordering::Equal {
            return p_cmp;
        }

        // 3. Inverse Cost (Minimize steps)
        other.normalized_cost.cmp(&self.normalized_cost)
    }
}

/// Compute evaluation metrics for a trajectory using u128 fixed-point
pub fn evaluate_path(traj: &AdmissibleTrajectory, max_depth: usize) -> PathEvaluation {
    // 1. Minimum Safety Margin (Bottleneck)
    let safety_bottleneck = traj
        .steps
        .iter()
        .map(|s| s.state_next.safety_margin())
        .fold(COH_PRECISION, |acc: u128, m: u128| acc.min(m));

    // 2. Alignment Index of last state (Target advanced score)
    let alignment = traj
        .steps
        .last()
        .map(|s| s.state_next.alignment_index())
        .unwrap_or(0);

    // 3. Normalized Cost (|\tau| / K_max)
    let normalized_cost = if max_depth == 0 {
        0
    } else {
        (traj.steps.len() as u128 * COH_PRECISION) / max_depth as u128
    };

    PathEvaluation {
        safety_bottleneck,
        progress: alignment,
        normalized_cost: normalized_cost.min(COH_PRECISION),
    }
}

/// Scalar weighted sum for UI display (Selection uses evaluate_path().cmp())
pub fn calculate_weighted_score(
    eval: &PathEvaluation,
    goal: u128,
    risk: u128,
    cost: u128,
    _uncertainty: u128,
) -> u128 {
    let p_part = (eval.progress * goal) / COH_PRECISION;
    let s_part = (eval.safety_bottleneck * risk) / COH_PRECISION;
    let c_part = (eval.normalized_cost * cost) / COH_PRECISION;

    p_part + s_part - c_part
}
