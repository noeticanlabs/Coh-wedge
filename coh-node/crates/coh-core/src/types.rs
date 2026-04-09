use serde::{Deserialize, Serialize};
pub use crate::reject::RejectCode;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub fn from_hex(hex: &str) -> Result<Self, RejectCode> {
        let bytes = hex::decode(hex).map_err(|_| RejectCode::RejectNumericParse)?;
        if bytes.len() != 32 {
            return Err(RejectCode::RejectNumericParse);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Decision {
    Accept,
    Reject,
}

// --- Wire Layer ---

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsWire {
    pub v_pre: String,
    pub v_post: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MicroReceiptWire {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub step_type: String,
    pub step_index: u64,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub state_hash_prev: String,
    pub state_hash_next: String,
    pub metrics: MetricsWire,
    pub spend: String,
    pub defect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabSummaryWire {
    pub state_hash_pre: String,
    pub state_hash_post: String,
    pub v_pre: String,
    pub v_post: String,
    pub spend: String,
    pub defect: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlabReceiptWire {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: String,
    pub policy_hash: String,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: String,
    pub chain_digest_next: String,
    pub merkle_root: String,
    pub summary: SlabSummaryWire,
}

// --- Runtime Layer ---

pub struct Metrics {
    pub v_pre: u128,
    pub v_post: u128,
}

pub struct MicroReceipt {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub step_type: String,
    pub step_index: u64,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub state_hash_prev: Hash32,
    pub state_hash_next: Hash32,
    pub metrics: Metrics,
    pub spend: u128,
    pub defect: u128,
}

pub struct SlabSummary {
    pub state_hash_pre: Hash32,
    pub state_hash_post: Hash32,
    pub v_pre: u128,
    pub v_post: u128,
    pub spend: u128,
    pub defect: u128,
}

pub struct SlabReceipt {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub range_start: u64,
    pub range_end: u64,
    pub micro_count: u64,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,
    pub merkle_root: Hash32,
    pub summary: SlabSummary,
}

// --- Prehash Layer (Alphabetized) ---

#[derive(Serialize)]
pub struct MetricsPrehash {
    pub v_post: String,
    pub v_pre: String,
}

#[derive(Serialize)]
pub struct MicroReceiptPrehash {
    pub canon_profile_hash: String,
    pub chain_digest_prev: String,
    pub defect: String,
    pub metrics: MetricsPrehash,
    pub object_id: String,
    pub policy_hash: String,
    pub schema_id: String,
    pub spend: String,
    pub state_hash_next: String,
    pub state_hash_prev: String,
    pub step_index: u64,
    pub step_type: String,
    pub version: u32,
}

// --- Result Layer ---

#[derive(Serialize)]
pub struct VerifyMicroResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
}

#[derive(Serialize)]
pub struct VerifyChainResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub failing_step: Option<u64>,
}

#[derive(Serialize)]
pub struct BuildSlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
    pub slab: Option<SlabReceiptWire>,
}

#[derive(Serialize)]
pub struct VerifySlabResult {
    pub decision: Decision,
    pub code: Option<RejectCode>,
}

// --- Conversions ---

fn parse_u128(s: &str) -> Result<u128, RejectCode> {
    s.parse::<u128>().map_err(|_| RejectCode::RejectNumericParse)
}

impl TryFrom<MetricsWire> for Metrics {
    type Error = RejectCode;
    fn try_from(w: MetricsWire) -> Result<Self, Self::Error> {
        Ok(Metrics {
            v_pre: parse_u128(&w.v_pre)?,
            v_post: parse_u128(&w.v_post)?,
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
            step_type: w.step_type,
            step_index: w.step_index,
            chain_digest_prev: Hash32::from_hex(&w.chain_digest_prev)?,
            chain_digest_next: Hash32::from_hex(&w.chain_digest_next)?,
            state_hash_prev: Hash32::from_hex(&w.state_hash_prev)?,
            state_hash_next: Hash32::from_hex(&w.state_hash_next)?,
            metrics: Metrics::try_from(w.metrics)?,
            spend: parse_u128(&w.spend)?,
            defect: parse_u128(&w.defect)?,
        })
    }
}
impl TryFrom<SlabSummaryWire> for SlabSummary {
    type Error = RejectCode;
    fn try_from(w: SlabSummaryWire) -> Result<Self, Self::Error> {
        Ok(SlabSummary {
            state_hash_pre: Hash32::from_hex(&w.state_hash_pre)?,
            state_hash_post: Hash32::from_hex(&w.state_hash_post)?,
            v_pre: parse_u128(&w.v_pre)?,
            v_post: parse_u128(&w.v_post)?,
            spend: parse_u128(&w.spend)?,
            defect: parse_u128(&w.defect)?,
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
            merkle_root: Hash32::from_hex(&w.merkle_root)?,
            summary: SlabSummary::try_from(w.summary)?,
        })
    }
}
