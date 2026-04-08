use crate::types::Hash32;
use sha2::{Sha256, Digest};

pub fn sha256(bytes: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}

pub fn domain_hash(tag: &[u8], payload: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(tag);
    hasher.update(payload);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}

pub fn update_chain_digest(prev: Hash32, canonical_receipt_bytes: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(crate::policy::CHAIN_TAG);
    hasher.update(&prev.0);
    hasher.update(canonical_receipt_bytes);
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    Hash32(arr)
}
