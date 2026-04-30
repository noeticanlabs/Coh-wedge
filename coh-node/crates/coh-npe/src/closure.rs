//! Lean Proof Closure Tracking
//!
//! Defines the formal closure status of a proof attempt.

use serde::{Deserialize, Serialize};

/// Status of a Lean proof closure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
pub enum LeanClosureStatus {
    /// Proof failed to build (syntax or type error)
    #[default]
    BuildFailed,
    /// Build passed but contains 'sorry'
    BuildPassedWithSorry,
    /// Build passed but contains 'admit'
    BuildPassedWithAdmit,
    /// Fully closed proof with no 'sorry' or 'admit'
    ClosedNoSorry,
    /// Error in the Lean toolchain or environment
    ToolchainError,
}

impl LeanClosureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeanClosureStatus::BuildFailed => "BuildFailed",
            LeanClosureStatus::BuildPassedWithSorry => "BuildPassedWithSorry",
            LeanClosureStatus::BuildPassedWithAdmit => "BuildPassedWithAdmit",
            LeanClosureStatus::ClosedNoSorry => "ClosedNoSorry",
            LeanClosureStatus::ToolchainError => "ToolchainError",
        }
    }

    /// Is the proof formally closed?
    pub fn is_closed(&self) -> bool {
        matches!(self, LeanClosureStatus::ClosedNoSorry)
    }

    /// Weight delta for PhaseLoom based on closure status
    pub fn weight_delta(&self) -> f64 {
        match self {
            LeanClosureStatus::ClosedNoSorry => 5.0,
            LeanClosureStatus::BuildPassedWithSorry => 0.5,
            LeanClosureStatus::BuildPassedWithAdmit => 0.2,
            LeanClosureStatus::BuildFailed => -1.0,
            LeanClosureStatus::ToolchainError => 0.0,
        }
    }
}
