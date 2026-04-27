use crate::types::{AttemptLogEntry, LedgerTimeEntry, TimeIndexState};
use coh_core::types::{Decision, Hash32};
use coh_core::reject::RejectCode;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CohTimeEngine {
    state: TimeIndexState,
    attempt_log: Vec<AttemptLogEntry>,
    ledger: Vec<LedgerTimeEntry>,
}

impl CohTimeEngine {
    pub fn new() -> Self {
        Self {
            state: TimeIndexState::default(),
            attempt_log: Vec::new(),
            ledger: Vec::new(),
        }
    }

    pub fn state(&self) -> &TimeIndexState {
        &self.state
    }

    pub fn apply_decision(
        &mut self,
        receipt_digest: Hash32,
        decision: Decision,
        error_code: Option<RejectCode>,
        state_hash_next: Option<Hash32>,
    ) -> (u64, u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let current_attempt = self.state.attempt_index;
        
        // Log the attempt
        self.attempt_log.push(AttemptLogEntry {
            attempt_index: current_attempt,
            timestamp,
            receipt_digest,
            decision,
            error_code,
        });

        // If accepted, advance ledger time
        if decision == Decision::Accept {
            if let Some(hash) = state_hash_next {
                self.ledger.push(LedgerTimeEntry {
                    accepted_index: self.state.accepted_index,
                    attempt_index: current_attempt,
                    timestamp,
                    state_hash_next: hash,
                });
                self.state.accepted_index += 1;
            }
        }

        // Always increment attempt index
        self.state.attempt_index += 1;

        (current_attempt, self.state.accepted_index)
    }

    pub fn get_ledger(&self) -> &[LedgerTimeEntry] {
        &self.ledger
    }

    pub fn get_attempt_log(&self) -> &[AttemptLogEntry] {
        &self.attempt_log
    }
}

impl Default for CohTimeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coh_core::types::Hash32;

    #[test]
    fn test_time_advancement() {
        let mut engine = CohTimeEngine::new();
        let dummy_digest = Hash32([0; 32]);
        let state_hash = Hash32([1; 32]);

        // First attempt - ACCEPT
        let (att1, acc1) = engine.apply_decision(dummy_digest, Decision::Accept, None, Some(state_hash));
        assert_eq!(att1, 0);
        assert_eq!(acc1, 1);
        assert_eq!(engine.state().attempt_index, 1);
        assert_eq!(engine.state().accepted_index, 1);

        // Second attempt - REJECT
        let (att2, acc2) = engine.apply_decision(dummy_digest, Decision::Reject, Some(RejectCode::RejectSchema), None);
        assert_eq!(att2, 1);
        assert_eq!(acc2, 1); // accepted index stays at 1
        assert_eq!(engine.state().attempt_index, 2);
        assert_eq!(engine.state().accepted_index, 1);

        // Third attempt - ACCEPT
        let (att3, acc3) = engine.apply_decision(dummy_digest, Decision::Accept, None, Some(state_hash));
        assert_eq!(att3, 2);
        assert_eq!(acc3, 2);
        assert_eq!(engine.state().attempt_index, 3);
        assert_eq!(engine.state().accepted_index, 2);
    }
}
