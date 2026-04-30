//! Governed Knowledge Formation Model
//! 
//! "GMI turns world knowledge into governed knowledge."

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeState {
    Proposed,
    Extracted,
    Supported,
    Verified,
    Operational,
    Volatile,
    Expired,
    Rejected,
    Superseded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationType {
    MathProof,
    FormalProof,
    CodeCompileTest,
    PhysicsResidual,
    DimensionalCheck,
    OperationalChecklist,
    SourceCitation,
    TimeSensitiveSource,
    PolicyGate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub claim: String,
    pub domain: String,
    pub state: KnowledgeState,
    pub verification_route: VerificationType,
    pub confidence: f64,
    pub receipt_id: Option<String>,
}

impl KnowledgeItem {
    pub fn new(id: String, claim: String, domain: String, verification_route: VerificationType) -> Self {
        Self {
            id,
            claim,
            domain,
            state: KnowledgeState::Proposed,
            verification_route,
            confidence: 0.0,
            receipt_id: None,
        }
    }

    pub fn mark_verified(&mut self, receipt_id: String) {
        self.state = KnowledgeState::Verified;
        self.confidence = 1.0;
        self.receipt_id = Some(receipt_id);
    }
}
