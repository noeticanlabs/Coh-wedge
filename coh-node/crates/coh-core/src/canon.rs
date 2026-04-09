use crate::types::{MicroReceipt, MicroReceiptPrehash, MetricsPrehash};
use crate::reject::RejectCode;

pub const EXPECTED_MICRO_SCHEMA_ID: &str = "coh.micro.v1";
pub const EXPECTED_MICRO_VERSION: u32 = 1;
pub const EXPECTED_CANON_PROFILE_HASH: &str = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

pub fn to_prehash_view(r: &MicroReceipt) -> MicroReceiptPrehash {
    MicroReceiptPrehash {
        canon_profile_hash: r.canon_profile_hash.to_hex(),
        chain_digest_prev: r.chain_digest_prev.to_hex(),
        defect: r.defect.to_string(),
        metrics: MetricsPrehash {
            v_post: r.metrics.v_post.to_string(),
            v_pre: r.metrics.v_pre.to_string(),
        },
        object_id: r.object_id.clone(),
        policy_hash: r.policy_hash.to_hex(),
        schema_id: r.schema_id.clone(),
        spend: r.spend.to_string(),
        state_hash_next: r.state_hash_next.to_hex(),
        state_hash_prev: r.state_hash_prev.to_hex(),
        step_index: r.step_index,
        step_type: r.step_type.clone(),
        version: r.version,
    }
}

pub fn to_canonical_json_bytes<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, RejectCode> {
    serde_json::to_vec(val).map_err(|_| RejectCode::RejectNumericParse)
}
