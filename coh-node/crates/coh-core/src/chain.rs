use crate::types::{MicroReceipt, SlabSummary, NodeContext};
use crate::reject::RejectCode;
use crate::verify_micro::verify_micro;

pub fn verify_chain(ctx: &NodeContext, receipts: &[MicroReceipt]) -> Result<SlabSummary, RejectCode> {
    if receipts.is_empty() {
        return Err(RejectCode::RejectSchema); // Or a specific empty error
    }

    let mut current_ctx = ctx.clone();
    let mut total_spend = 0i128;
    let mut total_defect = 0i128;

    for (i, r) in receipts.iter().enumerate() {
        // Linkage check (except for first which is checked against ctx)
        if i > 0 {
            if r.chain_digest_prev != receipts[i-1].chain_digest_next {
                return Err(RejectCode::RejectChainDigestPrev);
            }
            if r.state_hash_prev != receipts[i-1].state_hash_next {
                return Err(RejectCode::RejectStateHashLink);
            }
        }

        verify_micro(&current_ctx, r)?;
        
        total_spend += r.spend.0;
        total_defect += r.defect.0;
        
        // Update context for next step
        current_ctx.chain_digest_prev = r.chain_digest_next;
    }

    Ok(SlabSummary {
        state_hash_pre: receipts[0].state_hash_prev,
        state_hash_post: receipts.last().unwrap().state_hash_next,
        v_pre: receipts[0].metrics.v_pre,
        v_post: receipts.last().unwrap().metrics.v_post,
        spend: crate::types::QFixed(total_spend),
        defect: crate::types::QFixed(total_defect),
    })
}

pub fn aggregate_summary(receipts: &[MicroReceipt]) -> Result<SlabSummary, RejectCode> {
    if receipts.is_empty() {
        return Err(RejectCode::RejectSchema);
    }
    
    let mut total_spend = 0i128;
    let mut total_defect = 0i128;
    
    for r in receipts {
        total_spend += r.spend.0;
        total_defect += r.defect.0;
    }
    
    Ok(SlabSummary {
        state_hash_pre: receipts[0].state_hash_prev,
        state_hash_post: receipts.last().unwrap().state_hash_next,
        v_pre: receipts[0].metrics.v_pre,
        v_post: receipts.last().unwrap().metrics.v_post,
        spend: crate::types::QFixed(total_spend),
        defect: crate::types::QFixed(total_defect),
    })
}
