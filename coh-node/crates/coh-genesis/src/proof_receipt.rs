//! Proof Attempt Receipt Module
//!
//! Emits a micro-receipt per proof attempt from the NPE.
//! This connects the NPE proof-search loop to Coh receipts for:
//! - Search budget validation via existing coh-node verifier
//! - Goal-state embedding for proven_cache indexing
//! - Strategy learning through clustering similar goals

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Schema ID for proof attempt receipts
pub const PROOF_ATTEMPT_SCHEMA_ID: &str = "coh.receipt.proof_attempt.v1";
pub const PROOF_ATTEMPT_VERSION: &str = "1.0.0";

/// Goal state embedding - hash/embed of theorem statement for indexing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalEmbedding {
    /// SHA-256 hash of the theorem statement (canonical form)
    pub theorem_hash: String,
    /// Embedding vector for clustering (optional - for future strategy learning)
    pub embedding: Option<Vec<f64>>,
    /// Canonical theorem statement
    pub theorem_statement: String,
    /// Theorem name/identifier
    pub theorem_name: String,
}

impl GoalEmbedding {
    /// Create a new goal embedding from theorem statement
    pub fn new(theorem_name: &str, theorem_statement: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(theorem_statement.as_bytes());
        let result = hasher.finalize();
        let theorem_hash = hex::encode(result);

        Self {
            theorem_hash,
            embedding: None,
            theorem_statement: theorem_statement.to_string(),
            theorem_name: theorem_name.to_string(),
        }
    }

    /// Create with explicit embedding (for clustering)
    pub fn with_embedding(
        theorem_name: &str,
        theorem_statement: &str,
        embedding: Vec<f64>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(theorem_statement.as_bytes());
        let result = hasher.finalize();
        let theorem_hash = hex::encode(result);

        Self {
            theorem_hash,
            embedding: Some(embedding),
            theorem_statement: theorem_statement.to_string(),
            theorem_name: theorem_name.to_string(),
        }
    }
}

/// Search budget tracking
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchBudget {
    /// Total budget allocated for this proof search
    pub budget: u64,
    /// Amount spent so far
    pub spent: u64,
    /// Number of search steps taken
    pub steps: u64,
    /// Budget exhausted flag
    pub exhausted: bool,
}

impl SearchBudget {
    pub fn new(budget: u64) -> Self {
        Self {
            budget,
            spent: 0,
            steps: 0,
            exhausted: false,
        }
    }

    /// Increment search spend
    pub fn spend(&mut self, amount: u64) {
        self.spent = self.spent.saturating_add(amount);
        self.steps = self.steps.saturating_add(1);
        self.exhausted = self.spent >= self.budget;
    }

    /// Check if budget allows another step
    pub fn can_proceed(&self) -> bool {
        !self.exhausted && self.spent < self.budget
    }

    /// Remaining budget
    pub fn remaining(&self) -> u64 {
        self.budget.saturating_sub(self.spent)
    }
}

/// Proof attempt result
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofResult {
    /// Proof succeeded
    Proved,
    /// Proof failed (exhausted search)
    Failed,
    /// Search budget exceeded
    BudgetExceeded,
    /// Timeout or truncation
    Timeout,
    /// Incomplete proof (has gaps)
    Incomplete,
}

/// Proof attempt receipt - micro receipt per proof attempt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofAttemptReceipt {
    /// Schema identifier
    pub schema_id: String,
    /// Version
    pub version: String,
    /// Unique identifier for this proof attempt
    pub attempt_id: String,
    /// Goal state embedding for indexing
    pub goal_embedding: GoalEmbedding,
    /// Search budget tracking
    pub search_budget: SearchBudget,
    /// Proof result
    pub result: ProofResult,
    /// Genesis metrics: M(g), C(p), D(p)
    pub genesis_metrics: GenesisMetricsReceipt,
    /// Coherence metrics: V(pre), V(post)
    pub coherence_metrics: CoherenceMetricsReceipt,
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Parent chain digest (if chaining receipts)
    pub parent_digest: Option<String>,
    /// Chain digest of this receipt
    pub chain_digest: String,
    /// Lean verification details (if compiled)
    pub lean_details: Option<LeanVerificationDetails>,
}

/// Genesis metrics for proof attempt
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisMetricsReceipt {
    /// M(g): Initial disorder/complexity
    pub m_pre: u128,
    /// M(g'): Final disorder/complexity
    pub m_post: u128,
    /// C(p): Proof generation cost
    pub cost: u128,
    /// D(p): Generative slack
    pub slack: u128,
    /// Law of Genesis check: M(g') + C(p) <= M(g) + D(p)
    pub law_holds: bool,
}

/// Coherence metrics for proof attempt
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoherenceMetricsReceipt {
    /// V(y): Pre-proof risk valuation
    pub v_pre: u128,
    /// V(y'): Post-proof risk valuation  
    pub v_post: u128,
    /// Coherence holds: V(post) <= V(pre)
    pub coherence_holds: bool,
}

/// Lean verification details (from Lake compilation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeanVerificationDetails {
    /// Compilation success flag
    pub compiles: bool,
    /// Has sorry/admit in proof
    pub has_sorry: bool,
    /// Number of new axioms introduced
    pub new_axioms: usize,
    /// Build time in milliseconds
    pub build_time_ms: u64,
    /// Errors (if any)
    pub errors: Vec<String>,
    /// Warnings count
    pub warnings: usize,
    /// Statement unchanged flag
    pub statement_unchanged: bool,
    /// Forbidden imports used
    pub forbidden_imports: bool,
}

impl ProofAttemptReceipt {
    /// Create a new proof attempt receipt
    pub fn new(
        attempt_id: &str,
        goal_embedding: GoalEmbedding,
        search_budget: SearchBudget,
        result: ProofResult,
        genesis_metrics: GenesisMetricsReceipt,
        coherence_metrics: CoherenceMetricsReceipt,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Compute chain digest
        let mut hasher = Sha256::new();
        hasher.update(attempt_id.as_bytes());
        hasher.update(goal_embedding.theorem_hash.as_bytes());
        hasher.update(search_budget.spent.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        let chain_digest = hex::encode(hasher.finalize());

        Self {
            schema_id: PROOF_ATTEMPT_SCHEMA_ID.to_string(),
            version: PROOF_ATTEMPT_VERSION.to_string(),
            attempt_id: attempt_id.to_string(),
            goal_embedding,
            search_budget,
            result,
            genesis_metrics,
            coherence_metrics,
            timestamp,
            parent_digest: None,
            chain_digest,
            lean_details: None,
        }
    }

    /// Create with Lean verification details
    pub fn with_lean_details(mut self, details: LeanVerificationDetails) -> Self {
        self.lean_details = Some(details);
        self
    }

    /// Set parent digest for chaining
    pub fn with_parent_digest(mut self, parent: &str) -> Self {
        self.parent_digest = Some(parent.to_string());
        self
    }

    /// Convert to MicroReceiptWire format for coh-node verifier
    pub fn to_micro_receipt_wire(&self) -> ProofAttemptReceiptWire {
        ProofAttemptReceiptWire {
            schema_id: self.schema_id.clone(),
            version: self.version.clone(),
            attempt_id: self.attempt_id.clone(),
            theorem_hash: self.goal_embedding.theorem_hash.clone(),
            theorem_statement: self.goal_embedding.theorem_statement.clone(),
            theorem_name: self.goal_embedding.theorem_name.clone(),
            budget: self.search_budget.budget,
            spent: self.search_budget.spent,
            steps: self.search_budget.steps,
            result: format!("{:?}", self.result).to_lowercase(),
            m_pre: self.genesis_metrics.m_pre.to_string(),
            m_post: self.genesis_metrics.m_post.to_string(),
            cost: self.genesis_metrics.cost.to_string(),
            slack: self.genesis_metrics.slack.to_string(),
            law_holds: self.genesis_metrics.law_holds,
            v_pre: self.coherence_metrics.v_pre.to_string(),
            v_post: self.coherence_metrics.v_post.to_string(),
            coherence_holds: self.coherence_metrics.coherence_holds,
            timestamp: self.timestamp,
            parent_digest: self.parent_digest.clone(),
            chain_digest: self.chain_digest.clone(),
            lean_compiles: self.lean_details.as_ref().map(|d| d.compiles),
            lean_errors: self.lean_details.as_ref().map(|d| d.errors.join("; ")),
        }
    }

    pub fn to_legacy_micro_receipt_wire(&self) -> coh_core::types::MicroReceiptWire {
         coh_core::types::MicroReceiptWire {
            schema_id: self.schema_id.clone(),
            version: self.version.clone(),
            object_id: self.attempt_id.clone(),
            canon_profile_hash: "0".repeat(64),
            policy_hash: "0".repeat(64),
            step_index: self.search_budget.steps,
            step_type: Some("NPE_PROOF_ATTEMPT".to_string()),
            signatures: None,
            state_hash_prev: "0".repeat(64),
            state_hash_next: "0".repeat(64),
            chain_digest_prev: self.parent_digest.clone().unwrap_or_else(|| "0".repeat(64)),
            chain_digest_next: self.chain_digest.clone(),
            profile: coh_core::types::AdmissionProfile::CoherenceOnlyV1,
            metrics: coh_core::types::MetricsWire {
                v_pre: self.coherence_metrics.v_pre.to_string(),
                v_post: self.coherence_metrics.v_post.to_string(),
                spend: self.search_budget.spent.to_string(),
                defect: "0".to_string(),
                authority: "0".to_string(),
                m_pre: self.genesis_metrics.m_pre.to_string(),
                m_post: self.genesis_metrics.m_post.to_string(),
                c_cost: self.genesis_metrics.cost.to_string(),
                d_slack: self.genesis_metrics.slack.to_string(),
                ..Default::default()
            },
        }
    }
}

/// Wire format for JSON serialization (matches coh-node format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofAttemptReceiptWire {
    pub schema_id: String,
    pub version: String,
    pub attempt_id: String,
    pub theorem_hash: String,
    pub theorem_statement: String,
    pub theorem_name: String,
    pub budget: u64,
    pub spent: u64,
    pub steps: u64,
    pub result: String,
    pub m_pre: String,
    pub m_post: String,
    pub cost: String,
    pub slack: String,
    pub law_holds: bool,
    pub v_pre: String,
    pub v_post: String,
    pub coherence_holds: bool,
    pub timestamp: u64,
    pub parent_digest: Option<String>,
    pub chain_digest: String,
    pub lean_compiles: Option<bool>,
    pub lean_errors: Option<String>,
}

impl ProofAttemptReceipt {
    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let wire = self.to_micro_receipt_wire();
        serde_json::to_string_pretty(&wire)
    }
}

/// Proven cache for indexing proven theorems
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProvenCache {
    /// Map from theorem hash to receipt
    pub receipts: HashMap<String, ProofAttemptReceipt>,
    /// Map from theorem name to theorem hash
    pub name_to_hash: HashMap<String, String>,
}

impl ProvenCache {
    pub fn new() -> Self {
        Self {
            receipts: HashMap::new(),
            name_to_hash: HashMap::new(),
        }
    }

    /// Add a proven receipt to the cache
    pub fn insert(&mut self, receipt: ProofAttemptReceipt) {
        let hash = receipt.goal_embedding.theorem_hash.clone();
        let name = receipt.goal_embedding.theorem_name.clone();
        self.receipts.insert(hash.clone(), receipt);
        self.name_to_hash.insert(name, hash);
    }

    /// Look up by theorem hash
    pub fn get_by_hash(&self, hash: &str) -> Option<&ProofAttemptReceipt> {
        self.receipts.get(hash)
    }

    /// Look up by theorem name
    pub fn get_by_name(&self, name: &str) -> Option<&ProofAttemptReceipt> {
        self.name_to_hash
            .get(name)
            .and_then(|h| self.receipts.get(h))
    }

    /// Check if theorem is proven
    pub fn is_proven(&self, hash: &str) -> bool {
        self.receipts.contains_key(hash)
    }

    /// Get all proven theorem hashes
    pub fn proven_hashes(&self) -> Vec<String> {
        self.receipts.keys().cloned().collect()
    }

    /// Cluster similar goals by embedding distance (for strategy learning)
    pub fn cluster_similar(
        &self,
        target: &GoalEmbedding,
        _epsilon: f64,
    ) -> Vec<&ProofAttemptReceipt> {
        // Simple clustering by exact theorem hash match (embedding-based clustering for future)
        if let Some(r) = self.receipts.get(&target.theorem_hash) {
            vec![r]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_embedding() {
        let goal = GoalEmbedding::new("test_theorem", "∀x : ℕ, x + 0 = x");
        assert!(!goal.theorem_hash.is_empty());
        assert_eq!(goal.theorem_name, "test_theorem");
    }

    #[test]
    fn test_search_budget() {
        let mut budget = SearchBudget::new(100);
        assert!(budget.can_proceed());
        budget.spend(50);
        assert_eq!(budget.remaining(), 50);
        budget.spend(50);
        assert!(!budget.can_proceed());
    }

    #[test]
    fn test_proof_receipt() {
        let goal = GoalEmbedding::new("test_theorem", "∀x : ℕ, x + 0 = x");
        let mut budget = SearchBudget::new(100);
        budget.spend(50);

        let genesis = GenesisMetricsReceipt {
            m_pre: 100,
            m_post: 80,
            cost: 50,
            slack: 100,
            law_holds: true,
        };

        let coherence = CoherenceMetricsReceipt {
            v_pre: 50,
            v_post: 30,
            coherence_holds: true,
        };

        let receipt = ProofAttemptReceipt::new(
            "attempt-1",
            goal,
            budget,
            ProofResult::Proved,
            genesis,
            coherence,
        );

        assert_eq!(receipt.result, ProofResult::Proved);
        assert!(!receipt.chain_digest.is_empty());

        // Test JSON export
        let json = receipt.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_proven_cache() {
        let mut cache = ProvenCache::new();

        let goal = GoalEmbedding::new("test_theorem", "∀x : ℕ, x + 0 = x");
        let budget = SearchBudget::new(100);
        let genesis = GenesisMetricsReceipt::default();
        let coherence = CoherenceMetricsReceipt::default();

        let receipt = ProofAttemptReceipt::new(
            "attempt-1",
            goal.clone(),
            budget,
            ProofResult::Proved,
            genesis,
            coherence,
        );

        cache.insert(receipt.clone());

        assert!(cache.is_proven(&goal.theorem_hash));
        assert!(cache.get_by_name("test_theorem").is_some());
    }
}
