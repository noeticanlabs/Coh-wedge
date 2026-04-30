//! Shared traits for GMI Kernel tools.
//! 
//! "Same tool may share implementation, but mode determines authority."

use serde::{Deserialize, Serialize};

pub use coh_core::types::{ResourceCost, ToolAuthorityMode};

/// A tool report from an NPE-side exploratory run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeToolReport {
    pub tool_name: String,
    pub status: String,
    pub data: serde_json::Value,
    pub cost: ResourceCost,
}

/// A tool report from an RV-side certification run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RvToolReport {
    pub tool_name: String,
    pub accepted: bool,
    pub data: serde_json::Value,
    pub cost: ResourceCost,
}

/// Trait for NPE-side tools (TN)
pub trait NpeTool {
    fn name(&self) -> &'static str;
    fn mode(&self) -> ToolAuthorityMode { ToolAuthorityMode::Exploratory }
    fn run_soft(&self, content: &str) -> Result<NpeToolReport, String>;
}

/// Trait for RV-side tools (TR)
pub trait RvTool {
    fn name(&self) -> &'static str;
    fn mode(&self) -> ToolAuthorityMode { ToolAuthorityMode::Certification }
    fn run_cert(&self, claim: &str) -> Result<RvToolReport, String>;
}

pub trait NpeGenerator {
    type Context;
    fn generate(&self, seed: u64, index: usize, ctx: &Self::Context) -> Result<String, crate::engine::NpeError>;
}

pub trait NpeScorer {
    fn score(&self, proposal: &crate::engine::NpeProposal) -> Result<f64, crate::engine::NpeError>;
}

pub trait NpeVerifier {
    type Proof;
    fn verify(&self, proposal: &crate::engine::NpeProposal) -> Result<Self::Proof, crate::engine::NpeError>;
}
