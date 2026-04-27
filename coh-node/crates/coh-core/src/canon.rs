use crate::reject::RejectCode;
use crate::types::{MetricsPrehash, MicroReceipt, MicroReceiptPrehash};

pub const EXPECTED_MICRO_SCHEMA_ID: &str = "coh.receipt.micro.v1";
pub const EXPECTED_MICRO_VERSION: &str = "1.0.0";
pub const EXPECTED_SLAB_SCHEMA_ID: &str = "coh.receipt.slab.v1";
pub const EXPECTED_SLAB_VERSION: &str = "1.0.0";
pub const EXPECTED_CANON_PROFILE_HASH: &str =
    "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09";

pub fn to_prehash_view(r: &MicroReceipt) -> MicroReceiptPrehash {
    MicroReceiptPrehash {
        canon_profile_hash: r.canon_profile_hash.to_hex(),
        chain_digest_prev: r.chain_digest_prev.to_hex(),
        metrics: MetricsPrehash {
            authority: r.metrics.authority.to_string(),
            v_pre: r.metrics.v_pre.to_string(),
            v_post: r.metrics.v_post.to_string(),
            spend: r.metrics.spend.to_string(),
            defect: r.metrics.defect.to_string(),
        },
        object_id: r.object_id.clone(),
        policy_hash: r.policy_hash.to_hex(),
        schema_id: r.schema_id.clone(),
        signatures: r.signatures.clone(),
        state_hash_next: r.state_hash_next.to_hex(),
        state_hash_prev: r.state_hash_prev.to_hex(),
        step_index: r.step_index,
        step_type: r.step_type.clone(),
        version: r.version.clone(),
    }
}

pub fn to_canonical_json_bytes<T: serde::Serialize>(val: &T) -> Result<Vec<u8>, RejectCode> {
    // JCS Compliance Note:
    // Since our Prehash structs ensure alphabetical field order and we use
    // serde_json::to_vec (which omits whitespace), this is JCS-compatible
    // for our current schema (which uses Strings for all potentially
    // ambiguous numeric/special types).
    serde_json::to_vec(val).map_err(|_| RejectCode::RejectNumericParse)
}
