use crate::types::{SlabReceiptWire, SlabReceipt, VerifySlabResult, Decision, RejectCode};
use crate::math::CheckedMath;
use std::convert::TryFrom;

pub fn verify_slab(wire: SlabReceiptWire) -> VerifySlabResult {
    // 1. Wire to runtime
    let r = match SlabReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => return VerifySlabResult { decision: Decision::Reject, code: Some(e) },
    };

    // 2. Envelope checks
    if r.schema_id != "coh.slab.v1" {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectSchema) };
    }
    if r.version != 1 {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectVersion) };
    }

    // 3. Range sanity
    if r.micro_count == 0 {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectSchema) };
    }
    if r.range_end < r.range_start {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectSchema) };
    }
    
    // Exact count matches interval check:
    // (range_end - range_start + 1) == micro_count
    if (r.range_end - r.range_start + 1) != r.micro_count {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectSchema) };
    }

    // 4. Macro inequality
    // v_post + total_spend <= v_pre + total_defect
    let left_side = match r.summary.v_post.safe_add(r.summary.spend) {
        Ok(val) => val,
        Err(e) => return VerifySlabResult { decision: Decision::Reject, code: Some(e) },
    };
    let right_side = match r.summary.v_pre.safe_add(r.summary.defect) {
        Ok(val) => val,
        Err(e) => return VerifySlabResult { decision: Decision::Reject, code: Some(e) },
    };

    if left_side > right_side {
        return VerifySlabResult { decision: Decision::Reject, code: Some(RejectCode::RejectRiskBound) };
    }

    VerifySlabResult { decision: Decision::Accept, code: None }
}
