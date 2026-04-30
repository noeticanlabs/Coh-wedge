//! PhaseLoom Budget Logic

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseLoomBudget {
    pub memory_bytes: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub compression_ops: u64,
    pub latency_ms: u64,
    pub max_nodes: u64,
    pub max_entropy: f64,
    pub max_tension: f64,
}

impl Default for PhaseLoomBudget {
    fn default() -> Self {
        Self {
            memory_bytes: 10 * 1024 * 1024,
            read_ops: 10_000,
            write_ops: 10_000,
            compression_ops: 1_000,
            latency_ms: 200,
            max_nodes: 100_000,
            max_entropy: 10.0,
            max_tension: 100.0,
        }
    }
}
