use crate::types::{Hash32, MicroReceipt};
use crate::errors::CanonError;
use crate::canon::canonical_json_bytes;
use crate::hash::domain_hash;

pub fn micro_receipt_leaf(receipt: &MicroReceipt) -> Result<Hash32, CanonError> {
    let bytes = canonical_json_bytes(receipt)?;
    Ok(domain_hash(crate::policy::MERKLE_TAG, &bytes))
}

pub fn build_merkle_root(receipts: &[MicroReceipt]) -> Result<Hash32, CanonError> {
    if receipts.is_empty() {
        return Ok(Hash32([0u8; 32]));
    }
    let mut layer: Vec<Hash32> = receipts.iter()
        .map(|r| micro_receipt_leaf(r))
        .collect::<Result<Vec<_>, _>>()?;
    
    while layer.len() > 1 {
        let mut next_layer = Vec::with_capacity((layer.len() + 1) / 2);
        for chunk in layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&left.0);
            combined[32..].copy_from_slice(&right.0);
            next_layer.push(domain_hash(crate::policy::MERKLE_TAG, &combined));
        }
        layer = next_layer;
    }
    Ok(layer[0])
}

pub fn merkle_path(receipts: &[MicroReceipt], index: usize) -> Result<Vec<Hash32>, CanonError> {
    if index >= receipts.len() {
        return Err(CanonError::Overflow);
    }
    let mut layer: Vec<Hash32> = receipts.iter()
        .map(|r| micro_receipt_leaf(r))
        .collect::<Result<Vec<_>, _>>()?;
    
    let mut path = Vec::new();
    let mut curr_idx = index;
    
    while layer.len() > 1 {
        let sibling_idx = if curr_idx % 2 == 0 {
            if curr_idx + 1 < layer.len() { curr_idx + 1 } else { curr_idx }
        } else {
            curr_idx - 1
        };
        path.push(layer[sibling_idx]);
        
        let mut next_layer = Vec::new();
        for chunk in layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&left.0);
            combined[32..].copy_from_slice(&right.0);
            next_layer.push(domain_hash(crate::policy::MERKLE_TAG, &combined));
        }
        layer = next_layer;
        curr_idx /= 2;
    }
    Ok(path)
}

pub fn verify_merkle_path(leaf: Hash32, index: usize, path: &[Hash32], root: Hash32) -> bool {
    let mut current = leaf;
    let mut curr_idx = index;
    for sibling in path {
        let mut combined = [0u8; 64];
        if curr_idx % 2 == 0 {
            combined[..32].copy_from_slice(&current.0);
            combined[32..].copy_from_slice(&sibling.0);
        } else {
            combined[..32].copy_from_slice(&sibling.0);
            combined[32..].copy_from_slice(&current.0);
        }
        current = domain_hash(crate::policy::MERKLE_TAG, &combined);
        curr_idx /= 2;
    }
    current == root
}
