pub use crate::reject::RejectCode;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub fn from_hex(hex: &str) -> Result<Hash32, RejectCode> {
        if hex.len() != 64 {
            return Err(RejectCode::RejectNumericParse);
        }
        let bytes = hex::decode(hex).map_err(|_| RejectCode::RejectNumericParse)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&slice[..32]);
        Hash32(arr)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceCost {
    pub cpu_ms: u128,
    pub mem_bytes: u128,
    pub token_count: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolAuthorityMode {
    /// Exploratory mode: results are advisory, heuristics allowed.
    #[default]
    Exploratory,
    /// Certification mode: results are deterministic, must be auditable.
    Certification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decision {
    Accept,
    Reject,
    SlabBuilt,
    TerminalSuccess,
    TerminalFailure,
    AbortBudget,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionProfile {
    #[default]
    CoherenceOnlyV1,
    FormationV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsWire {
    pub v_pre: String,
    pub v_post: String,
    pub spend: String,
    pub defect: String,
    #[serde(default = "default_authority")]
    pub authority: String,
    // Genesis Metrics (Forward Generation)
    #[serde(default)]
    pub m_pre: String,
    #[serde(default)]
    pub m_post: String,
    #[serde(default)]
    pub c_cost: String,
    #[serde(default)]
    pub d_slack: String,
    #[serde(default)]
    pub projection_hash: String,
    // PhaseLoom Ecology
    #[serde(default)]
    pub pl_tau: String,
    #[serde(default)]
    pub pl_budget: String,
    #[serde(default)]
    pub pl_provenance: String,
}

impl Default for MetricsWire {
    fn default() -> Self {
        Self {
            v_pre: "0".to_string(),
            v_post: "0".to_string(),
            spend: "0".to_string(),
            defect: "0".to_string(),
            authority: "0".to_string(),
            m_pre: "0".to_string(),
            m_post: "0".to_string(),
            c_cost: "0".to_string(),
            d_slack: "0".to_string(),
            projection_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            pl_tau: "0".to_string(),
            pl_budget: "0".to_string(),
            pl_provenance: "EXT".to_string(),
        }
    }
}

fn default_authority() -> String {
    "0".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignatureWire {
    pub signature: String,
    pub signer: String,
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MicroReceiptWire {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<SignatureWire>>,
    pub state_hash_prev: String,
    pub state_hash_next: String,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    #[serde(default)]
    pub profile: AdmissionProfile,
    pub metrics: MetricsWire,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabSummaryWire {
    pub total_spend: String,
    pub total_defect: String,
    pub v_pre_first: String,
    pub v_post_last: String,
    #[serde(default = "default_authority")]
    pub authority: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabReceiptWire {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub state_hash_first: String,
    pub state_hash_last: String,
    pub merkle_root: String,
    pub summary: SlabSummaryWire,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metrics {
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
    pub authority: u128,
    // Genesis Metrics (Forward Generation)
    pub m_pre: u128,
    pub m_post: u128,
    pub c_cost: u128,
    pub d_slack: u128,
    pub projection_hash: Hash32,
    // PhaseLoom Ecology
    pub pl_tau: u64,
    pub pl_budget: u128,
    pub pl_provenance: String,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            v_pre: 0,
            v_post: 0,
            spend: 0,
            defect: 0,
            authority: 0,
            m_pre: 0,
            m_post: 0,
            c_cost: 0,
            d_slack: 0,
            projection_hash: Hash32([0; 32]),
            pl_tau: 0,
            pl_budget: 0,
            pl_provenance: "EXT".to_string(),
        }
    }
}

/// A Certified Morphism in the Coh Category.
/// Mirrors the Slack2Cell and CertifiedMorphism structures in Lean.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CertifiedMorphism {
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
    pub authority: u128,
    // Genesis Metrics
    pub m_pre: u128,
    pub m_post: u128,
    pub c_cost: u128,
    pub d_slack: u128,
}

impl CertifiedMorphism {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        v_pre: u128,
        v_post: u128,
        spend: u128,
        defect: u128,
        authority: u128,
        m_pre: u128,
        m_post: u128,
        c_cost: u128,
        d_slack: u128,
    ) -> Self {
        Self {
            v_pre,
            v_post,
            spend,
            defect,
            authority,
            m_pre,
            m_post,
            c_cost,
            d_slack,
        }
    }

    /// The fundamental Coherence inequality: V_post + spend <= V_pre + defect + authority
    pub fn is_certified(&self) -> bool {
        let lhs = self.v_post.saturating_add(self.spend);
        let rhs = self
            .v_pre
            .saturating_add(self.defect)
            .saturating_add(self.authority);
        lhs <= rhs
    }

    /// The Law of Genesis: M(g') + C(p) <= M(g) + D(p)
    pub fn is_genesis_admissible(&self) -> bool {
        let lhs = self.m_post.checked_add(self.c_cost);
        let rhs = self.m_pre.checked_add(self.d_slack);
        match (lhs, rhs) {
            (Some(l), Some(r)) => l <= r,
            _ => false, // Overflow rejects
        }
    }

    /// Intersection of Genesis and Coherence
    pub fn is_formation_admissible(&self) -> bool {
        self.is_certified() && self.is_genesis_admissible()
    }

    /// Compose with another certified morphism (f ; g)
    /// Cost additivity: spend = spend_f + spend_g, defect = defect_f + defect_g
    pub fn compose(&self, other: &Self) -> Option<Self> {
        // v_post of first must match v_pre of second (simplified object match)
        if self.v_post != other.v_pre {
            return None;
        }

        let total_spend = self.spend.checked_add(other.spend)?;
        let total_defect = self.defect.checked_add(other.defect)?;
        let total_authority = self.authority.checked_add(other.authority)?;

        // Genesis composition (Additivity assumption for V1)
        let total_cost = self.c_cost.checked_add(other.c_cost)?;
        let total_slack = self.d_slack.checked_add(other.d_slack)?;

        Some(Self {
            v_pre: self.v_pre,
            v_post: other.v_post,
            spend: total_spend,
            defect: total_defect,
            authority: total_authority,
            m_pre: self.m_pre,
            m_post: other.m_post,
            c_cost: total_cost,
            d_slack: total_slack,
        })
    }
}

pub struct MicroReceipt {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub signatures: Option<Vec<SignatureWire>>,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub profile: AdmissionProfile,
    pub metrics: Metrics,
}

pub struct SlabSummary {
    pub total_spend: u128,
    pub total_defect: u128,
    pub v_pre_first: u128,
    pub v_post_last: u128,
    pub authority: u128,
}

pub struct SlabReceipt {
    pub schema_id: String,
    pub version: String,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub state_hash_first: Hash32,
    pub state_hash_last: Hash32,
    pub merkle_root: Hash32,
    pub summary: SlabSummary,
}

#[derive(Serialize)]
pub struct MetricsPrehash {
    pub authority: String,
    pub defect: String,
    pub spend: String,
    pub m_pre: String,
    pub m_post: String,
    pub v_post: String,
    pub v_pre: String,
    pub c_cost: String,
    pub d_slack: String,
    pub projection_hash: String,
    pub pl_tau: u64,
    pub pl_budget: String,
    pub pl_provenance: String,
}

#[derive(Serialize)]
pub struct MicroReceiptPrehash {
    pub canon_profile_hash: String,
    pub chain_digest_prev: String,
    pub metrics: MetricsPrehash,
    pub object_id: String,
    pub policy_hash: String,
    pub profile: String,
    pub schema_id: String,
    pub state_hash_next: String,
    pub state_hash_prev: String,
    pub step_index: u64,
    pub step_type: Option<String>,
    pub version: String,
}

#[derive(Serialize, Debug)]
pub struct VerifyMicroResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_digest_next: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct VerifyChainResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    pub steps_verified: u64,
    pub first_step_index: u64,
    pub last_step_index: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_chain_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_step_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps_verified_before_failure: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct BuildSlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_end: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slab: Option<SlabReceiptWire>,
}

#[derive(Serialize, Debug)]
pub struct VerifySlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub message: String,
    pub range_start: u64,
    pub range_end: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_root: Option<String>,
}

fn parse_u128(s: &str) -> Result<u128, RejectCode> {
    s.parse::<u128>()
        .map_err(|_| RejectCode::RejectNumericParse)
}

impl TryFrom<MetricsWire> for Metrics {
    type Error = RejectCode;
    fn try_from(w: MetricsWire) -> Result<Self, Self::Error> {
        Ok(Metrics {
            v_pre: parse_u128(&w.v_pre)?,
            v_post: parse_u128(&w.v_post)?,
            spend: parse_u128(&w.spend)?,
            defect: parse_u128(&w.defect)?,
            authority: parse_u128(&w.authority)?,
            m_pre: parse_u128(&w.m_pre)?,
            m_post: parse_u128(&w.m_post)?,
            c_cost: parse_u128(&w.c_cost)?,
            d_slack: parse_u128(&w.d_slack)?,
            projection_hash: Hash32::from_hex(&w.projection_hash)?,
            pl_tau: w.pl_tau.parse::<u64>().map_err(|_| RejectCode::RejectNumericParse)?,
            pl_budget: parse_u128(&w.pl_budget)?,
            pl_provenance: w.pl_provenance,
        })
    }
}

impl TryFrom<MicroReceiptWire> for MicroReceipt {
    type Error = RejectCode;
    fn try_from(w: MicroReceiptWire) -> Result<Self, Self::Error> {
        Ok(MicroReceipt {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)?,
            step_index: w.step_index,
            step_type: w.step_type,
            signatures: w.signatures,
            state_hash_prev: Hash32::from_hex(&w.state_hash_prev)?,
            state_hash_next: Hash32::from_hex(&w.state_hash_next)?,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)?,
            profile: w.profile,
            metrics: Metrics::try_from(w.metrics)?,
        })
    }
}

impl TryFrom<SlabSummaryWire> for SlabSummary {
    type Error = RejectCode;
    fn try_from(w: SlabSummaryWire) -> Result<Self, Self::Error> {
        Ok(SlabSummary {
            total_spend: parse_u128(&w.total_spend)?,
            total_defect: parse_u128(&w.total_defect)?,
            v_pre_first: parse_u128(&w.v_pre_first)?,
            v_post_last: parse_u128(&w.v_post_last)?,
            authority: parse_u128(&w.authority)?,
        })
    }
}

impl TryFrom<SlabReceiptWire> for SlabReceipt {
    type Error = RejectCode;
    fn try_from(w: SlabReceiptWire) -> Result<Self, Self::Error> {
        Ok(SlabReceipt {
            schema_id: w.schema_id,
            version: w.version,
            object_id: w.object_id,
            canon_profile_hash: Hash32::from_hex(&w.canon_profile_hash)?,
            policy_hash: Hash32::from_hex(&w.policy_hash)?,
            range_start: w.range_start,
            range_end: w.range_end,
            micro_count: w.micro_count,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)?,
            state_hash_first: Hash32::from_hex(&w.state_hash_first)?,
            state_hash_last: Hash32::from_hex(&w.state_hash_last)?,
            merkle_root: Hash32::from_hex(&w.merkle_root)?,
            summary: SlabSummary::try_from(w.summary)?,
        })
    }
}
