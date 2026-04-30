//! Trajectory Layer - V3 Geometry
//!
//! Implements the distance metric d(x, y) = inf { delta(tau) | tau : x -> y }

use crate::types::Hash32;
use serde::{Deserialize, Serialize};

/// A projection \Pi(\tau) = R
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Projection {
    pub record_hash: Hash32,
    pub domain_id: String,
}

/// A certified envelope \hat{\delta}(R)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeCertificate {
    pub projection_hash: Hash32,
    pub delta_hat: u128,
    pub verifier_signature: Vec<u8>,
}

/// A state in the trajectory graph
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateNode {
    pub hash: Hash32,
    pub potential: u128,
}

/// A transition between states with its associated metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub from: StateNode,
    pub to: StateNode,
    pub delta: u128,
    pub delta_hat: u128, // Certifiable defect (upper bound)
    pub projection_hash: Hash32,
    pub certificate_hash: Hash32,
    pub step_type: Option<String>,
}

/// An observable sequence of projections (Visible Trace)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservableTrace {
    pub projections: Vec<Projection>,
    pub total_delta_hat: u128,
}

/// A path of transitions (Trace)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trajectory {
    pub steps: Vec<Transition>,
}

impl Trajectory {
    /// Calculate the total raw defect of the trajectory
    pub fn total_defect(&self) -> u128 {
        self.steps.iter().map(|t| t.delta).sum()
    }

    /// Calculate the total certified defect d_hat(x, y)
    pub fn total_certified_defect(&self) -> u128 {
        self.steps.iter().map(|t| t.delta_hat).sum()
    }

    /// Project the trajectory into an observable trace
    pub fn project(&self) -> ObservableTrace {
        let projections = self.steps.iter().map(|t| Projection {
            record_hash: t.projection_hash,
            domain_id: t.step_type.clone().unwrap_or_else(|| "default".to_string()),
        }).collect();
        
        ObservableTrace {
            projections,
            total_delta_hat: self.total_certified_defect(),
        }
    }
}
