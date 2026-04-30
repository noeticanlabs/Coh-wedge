use coh_core::types::Hash32;
use coh_core::rv_kernel::RvDecision;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub proposal_id: String,
    pub prev_tip: Hash32,
    pub decision: RvDecision,
    pub timestamp: u64,
}

pub struct SimpleLedger {
    pub tip: Hash32,
    pub history: Vec<Receipt>,
}

impl SimpleLedger {
    pub fn new() -> Self {
        Self {
            tip: Hash32::default(),
            history: Vec::new(),
        }
    }

    pub fn append(&mut self, proposal_id: &str, decision: RvDecision) -> Result<Hash32, String> {
        let receipt = Receipt {
            proposal_id: proposal_id.to_string(),
            prev_tip: self.tip.clone(),
            decision,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs(),
        };

        // Update tip by hashing the receipt
        let mut hasher = Sha256::new();
        hasher.update(self.tip.as_bytes());
        hasher.update(receipt.proposal_id.as_bytes());
        hasher.update(serde_json::to_vec(&receipt.decision).map_err(|e| e.to_string())?);
        
        let new_tip = Hash32::from_slice(&hasher.finalize());
        self.tip = new_tip.clone();
        self.history.push(receipt);

        Ok(new_tip)
    }
}

impl Default for SimpleLedger {
    fn default() -> Self {
        Self::new()
    }
}
