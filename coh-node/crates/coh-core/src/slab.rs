use crate::types::{MicroReceipt, SlabReceipt, NodeContext, SlabSummary};
use crate::reject::RejectCode;
use crate::errors::CanonError;
use crate::merkle::build_merkle_root;
use crate::chain::aggregate_summary;

pub fn build_slab(
    ctx: &NodeContext,
    receipts: &[MicroReceipt],
    range_start: u64,
    range_end: u64,
) -> Result<SlabReceipt, CanonError> {
    let summary = aggregate_summary(receipts).map_err(|_| CanonError::Overflow)?;
    let merkle_root = build_merkle_root(receipts)?;
    
    Ok(SlabReceipt {
        schema_id: crate::policy::SLAB_SCHEMA_ID.to_string(),
        version: crate::policy::VERSION,
        object_id: ctx.object_id.clone(),
        canon_profile_hash: ctx.canon_profile_hash,
        policy_hash: ctx.policy_hash,
        range_start,
        range_end,
        chain_digest_prev: receipts[0].chain_digest_prev,
        chain_digest_next: receipts.last().unwrap().chain_digest_next,
        merkle_root,
        summary,
    })
}

pub fn verify_slab(ctx: &NodeContext, slab: &SlabReceipt) -> Result<(), RejectCode> {
    if slab.schema_id != crate::policy::SLAB_SCHEMA_ID {
        return Err(RejectCode::RejectSchema);
    }
    if slab.version != crate::policy::VERSION {
        return Err(RejectCode::RejectVersion);
    }
    if slab.canon_profile_hash != ctx.canon_profile_hash {
        return Err(RejectCode::RejectCanonProfile);
    }
    if slab.object_id != ctx.object_id {
        return Err(RejectCode::RejectObjectId);
    }
    if slab.policy_hash != ctx.policy_hash {
        return Err(RejectCode::RejectPolicyHash);
    }
    if slab.chain_digest_prev != ctx.chain_digest_prev {
        return Err(RejectCode::RejectChainDigestPrev);
    }
    
    // Check risk bound on macro summary
    let v_pre = slab.summary.v_pre.0;
    let v_post = slab.summary.v_post.0;
    let spend = slab.summary.spend.0;
    let defect = slab.summary.defect.0;
    
    if v_post + spend > v_pre + defect {
        return Err(RejectCode::RejectRiskBound);
    }

    Ok(())
}
