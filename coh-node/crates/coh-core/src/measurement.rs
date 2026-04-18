use crate::math::{CheckedMath, MathResult};
use crate::types::{Hash32, MicroReceipt};
use std::collections::HashMap;

/// A Measurement in Coh is a verification-preserving morphism (CohHom) 
/// between governed systems.
/// 
/// It must preserve:
/// 1. Structural regularity (Step validity)
/// 2. Quantitative regularity (Oplax / Dissipation)
pub trait Measurement {
    /// Maps a single verified step from system A to system B.
    /// Returns None if the mapping violates validity preservation.
    fn map_step(
        &self,
        pre: &Hash32,
        receipt: &MicroReceipt,
        post: &Hash32,
    ) -> Option<(Hash32, MicroReceipt, Hash32)>;
}

/// Calculate the cost of a receipt based on dissipated resources (spend + defect).
/// This defines the quantitative regularity for oplax mappings.
pub fn step_cost(r: &MicroReceipt) -> MathResult<u128> {
    r.metrics.spend.safe_add(r.metrics.defect)
}

/// Calculate the total cumulative cost of a trace (chain of receipts).
pub fn trace_cost(chain: &[MicroReceipt]) -> MathResult<u128> {
    let mut total = 0u128;
    for r in chain {
        total = total.safe_add(step_cost(r)?)?;
    }
    Ok(total)
}

/// Maps a verified chain along a measurement (DynMap functor).
/// Enforces chain continuity invariants: post_i == pre_{i+1}.
pub fn map_chain<M: Measurement>(m: &M, chain: &[MicroReceipt]) -> Option<Vec<MicroReceipt>> {
    let mut result = Vec::with_capacity(chain.len());
    
    for r in chain {
        let (mapped_pre, mapped_r, mapped_post) = m.map_step(&r.state_hash_prev, r, &r.state_hash_next)?;
        
        // Safety: verify continuity invariant in the mapped dynamics
        if let Some(prev) = result.last() as Option<&MicroReceipt> {
            if prev.state_hash_next != mapped_pre {
                return None; // Mapping breaks categorical composition
            }
        }
        
        // Ensure the mapped receipt reflects the mapped states
        if mapped_r.state_hash_prev != mapped_pre || mapped_r.state_hash_next != mapped_post {
            return None;
        }
        
        result.push(mapped_r);
    }
    
    Some(result)
}

/// Verify the Oplax dissipation constraint: 
/// The cost of observable dynamics cannot exceed the cost of hidden dynamics.
pub fn verify_chain_dissipation<M: Measurement>(m: &M, source_chain: &[MicroReceipt]) -> bool {
    let target_chain = match map_chain(m, source_chain) {
        Some(c) => c,
        None => return false,
    };
    
    let source_cost = match trace_cost(source_chain) {
        Ok(c) => c,
        Err(_) => return false,
    };
    
    let target_cost = match trace_cost(&target_chain) {
        Ok(c) => c,
        Err(_) => return false,
    };
    
    target_cost <= source_cost
}

/// Collapsed state information.
pub struct CollapseInfo {
    pub target_hash: Hash32,
    pub source_hashes: Vec<Hash32>,
}

/// Detect non-faithfulness (collapse) instances in a set of traces.
/// Returns a map of target states that are reachable from distinct source states.
pub fn detect_collapse<M: Measurement>(m: &M, traces: &[Vec<MicroReceipt>]) -> Vec<CollapseInfo> {
    let mut mapping: HashMap<Hash32, Vec<Hash32>> = HashMap::new();
    
    for chain in traces {
        for r in chain {
            let s_pre = r.state_hash_prev;
            if let Some((t_pre, _, _)) = m.map_step(&s_pre, r, &r.state_hash_next) {
                let entry = mapping.entry(t_pre).or_default();
                if !entry.contains(&s_pre) {
                    entry.push(s_pre);
                }
            }
        }
    }
    
    mapping.into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(target, sources)| CollapseInfo { target_hash: target, source_hashes: sources })
        .collect()
}
