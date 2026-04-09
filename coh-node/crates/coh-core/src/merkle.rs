use crate::types::Hash32;
use crate::hash::compute_merkle_inner;

pub fn build_merkle_root(leaves: &[Hash32]) -> Hash32 {
    if leaves.is_empty() {
        return Hash32([0u8; 32]);
    }
    let mut layer = leaves.to_vec();
    
    while layer.len() > 1 {
        let mut next_layer = Vec::with_capacity((layer.len() + 1) / 2);
        for chunk in layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
            next_layer.push(compute_merkle_inner(left, right));
        }
        layer = next_layer;
    }
    layer[0]
}
