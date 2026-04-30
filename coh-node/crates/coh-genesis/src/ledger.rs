use coh_core::types::{Hash32, VerifierClaim};
use coh_core::rv_kernel::RvDecision;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub schema_version: String,
    pub domain_tag: String,
    pub proposal_id: String,
    pub prev_tip: Hash32,
    pub claim: VerifierClaim,
    pub decision: RvDecision,
    pub timestamp: u64,
    pub sequence: u64,
    
    // Lexicon Canon Compliance
    pub lexicon_terms_used: Vec<String>,
    pub namespaces: Vec<String>,
    pub layers: Vec<String>,
    pub residuals: serde_json::Value,
    pub failure_class: Option<String>,
    
    // Physical/GMI Context
    pub budget_deltas: serde_json::Value,
    pub causal_class: Option<String>,
    pub branch_weight: Option<f64>,
    pub post_state_hash: Option<Hash32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub payload: ReceiptPayload,
    pub hash: Hash32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleLedger {
    pub tip: Hash32,
    pub sequence: u64,
    pub history: Vec<Receipt>,
}

impl SimpleLedger {
    pub fn new() -> Self {
        Self {
            tip: Hash32::default(),
            sequence: 0,
            history: Vec::new(),
        }
    }

    pub fn append(&mut self, proposal_id: &str, claim: VerifierClaim, decision: RvDecision) -> Result<Hash32, String> {
        let payload = ReceiptPayload {
            schema_version: "GMI_V1".into(),
            domain_tag: "GMI_RECEIPT".into(),
            proposal_id: proposal_id.to_string(),
            prev_tip: self.tip.clone(),
            claim,
            decision,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs(),
            sequence: self.sequence,
            
            lexicon_terms_used: vec![],
            namespaces: vec![],
            layers: vec![],
            residuals: serde_json::Value::Null,
            failure_class: None,
            budget_deltas: serde_json::Value::Null,
            causal_class: None,
            branch_weight: None,
            post_state_hash: None,
        };

        // Canonical hash of the payload
        let payload_bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        
        let new_tip = Hash32::from_slice(&hasher.finalize());
        self.tip = new_tip.clone();
        self.sequence = self.sequence.saturating_add(1);
        
        self.history.push(Receipt {
            payload,
            hash: new_tip.clone(),
        });

        Ok(new_tip)
    }
}

impl Default for SimpleLedger {
    fn default() -> Self {
        Self::new()
    }
}
