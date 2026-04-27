use serde::{Deserialize, Serialize};
use coh_core::types::{Decision, Hash32};
use coh_core::reject::RejectCode;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttemptLogEntry {
    pub attempt_index: u64,
    pub timestamp: u64,
    pub receipt_digest: Hash32,
    pub decision: Decision,
    pub error_code: Option<RejectCode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerTimeEntry {
    pub accepted_index: u64,
    pub attempt_index: u64,
    pub timestamp: u64,
    pub state_hash_next: Hash32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimeIndexState {
    pub attempt_index: u64,
    pub accepted_index: u64,
}
