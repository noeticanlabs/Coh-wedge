//! RV-side Tools (TR)
//! 
//! "RV tools certify or reject projected claims."

use crate::rv_kernel::RvKernel;
use serde_json::json;

/// Placeholder for the Law Checker tool
pub struct LawChecker;

impl LawChecker {
    pub fn check(&self, kernel: &RvKernel, next_v: u128, spend: u128, prev_v: u128, defect: u128) -> bool {
        kernel.is_admissible(next_v, spend, prev_v, defect)
    }
}

/// Placeholder for the Resource Gate tool
pub struct ResourceGate;

impl ResourceGate {
    pub fn check(&self, kernel: &RvKernel, estimated_cost: u64) -> bool {
        kernel.can_verify_safely(estimated_cost)
    }
}

/// RV Tool Reports
pub fn generate_law_report(accepted: bool, margin: f64) -> serde_json::Value {
    json!({
        "tool": "law_checker",
        "accepted": accepted,
        "coherence_margin": margin,
    })
}
