use crate::types::Hash32;
use sha2::{Digest, Sha256};

pub const DIGEST_DOMAIN_TAG: &[u8] = b"COH_V1_CHAIN";
pub const MERKLE_DOMAIN_TAG: &[u8] = b"COH_V1_MERKLE";
pub const PROJECTION_DOMAIN_TAG: &[u8] = b"COH_V3_PROJECTION";

pub fn sha256(bytes: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}

pub fn compute_chain_digest(prev_digest: Hash32, canonical_json_bytes: &[u8]) -> Hash32 {
    // SHA256(DIGEST_DOMAIN_TAG || "|" || chain_digest_prev || "|" || canonical_json(prehash_view))
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_DOMAIN_TAG);
    hasher.update(b"|");
    hasher.update(prev_digest.0);
    hasher.update(b"|");
    hasher.update(canonical_json_bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}

pub fn compute_merkle_inner(left: Hash32, right: Hash32) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(MERKLE_DOMAIN_TAG);
    hasher.update(b"|");
    hasher.update(left.0);
    hasher.update(b"|");
    hasher.update(right.0);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}

pub fn compute_projection_hash(canonical_json_bytes: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(PROJECTION_DOMAIN_TAG);
    hasher.update(b"|");
    hasher.update(canonical_json_bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}
