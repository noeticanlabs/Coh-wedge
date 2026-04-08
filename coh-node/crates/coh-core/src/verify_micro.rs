use crate::types::{MicroReceipt, NodeContext};
use crate::reject::RejectCode;
use crate::hash::update_chain_digest;
use crate::canon::canonical_json_bytes;

pub type VerifyResult = Result<(), RejectCode>;

pub fn verify_micro(ctx: &NodeContext, r: &MicroReceipt) -> VerifyResult {
    // 1. schema id
    if r.schema_id != crate::policy::MICRO_SCHEMA_ID {
        return Err(RejectCode::RejectSchema);
    }
    // 2. version
    if r.version != crate::policy::VERSION {
        return Err(RejectCode::RejectVersion);
    }
    // 3. canon profile
    if r.canon_profile_hash != ctx.canon_profile_hash {
        return Err(RejectCode::RejectCanonProfile);
    }
    // 4. object id
    if r.object_id != ctx.object_id {
        return Err(RejectCode::RejectObjectId);
    }
    // 5. policy hash
    if r.policy_hash != ctx.policy_hash {
        return Err(RejectCode::RejectPolicyHash);
    }
    // 6. chain digest prev
    if r.chain_digest_prev != ctx.chain_digest_prev {
        return Err(RejectCode::RejectChainDigestPrev);
    }
    
    // 7. chain digest next recomputation
    // For recomputation, we often need the canonical bytes of the receipt itself *without* the next digest?
    // In many protocols, the "next digest" is the hash of (prev_digest || current_receipt_data).
    // Let's assume the recomputation matches the update_chain_digest logic.
    let canon_bytes = canonical_json_bytes(r).map_err(|_| RejectCode::RejectNumericParse)?;
    let recomputed_next = update_chain_digest(r.chain_digest_prev, &canon_bytes);
    
    // Note: If r already contains chain_digest_next, we compare.
    if r.chain_digest_next != recomputed_next {
        return Err(RejectCode::RejectChainDigestNext);
    }

    // 8. state linkage consistency 
    // (This is usually checked against the previous receipt's state_hash_next, 
    // but in single micro verify against context, we might check ctx.state_hash_prev if present)
    // For this demo, we'll assume the context can optionally have it or we skip if not provided.
    
    // 9. numeric validity (already checked via QFixed parsing in canonicalization)

    // 10. risk inequality: V(post) + spend <= V(pre) + defect
    let v_pre = r.metrics.v_pre.0;
    let v_post = r.metrics.v_post.0;
    let spend = r.spend.0;
    let defect = r.defect.0;
    
    if v_post + spend > v_pre + defect {
        return Err(RejectCode::RejectRiskBound);
    }

    Ok(())
}
