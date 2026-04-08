use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QFixed(pub i128);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeContext {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,
    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Option<Hash32>,
    pub merkle_root: Option<Hash32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryMetrics {
    pub v_pre: QFixed,
    pub v_post: QFixed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub metrics: BoundaryMetrics,
    pub spend: QFixed,
    pub defect: QFixed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlabSummary {
    pub state_hash_pre: Hash32,
    pub state_hash_post: Hash32,
    pub v_pre: QFixed,
    pub v_post: QFixed,
    pub spend: QFixed,
    pub defect: QFixed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlabReceipt {
    pub schema_id: String,
    pub version: u32,
    pub object_id: String,
    pub canon_profile_hash: Hash32,
    pub policy_hash: Hash32,

    pub range_start: u64,
    pub range_end: u64,

    pub chain_digest_prev: Hash32,
    pub chain_digest_next: Hash32,

    pub merkle_root: Hash32,
    pub summary: SlabSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeOpening {
    pub index: usize,
    pub receipt: MicroReceipt,
    pub merkle_path: Vec<Hash32>,
}
