use crate::types::{MicroReceiptWire, VerifyChainResult, Decision, RejectCode};
use crate::verify_micro::verify_micro;

pub fn verify_chain(receipts: Vec<MicroReceiptWire>) -> VerifyChainResult {
    if receipts.is_empty() {
        return VerifyChainResult {
            decision: Decision::Accept,
            code: None,
            failing_step: None,
        };
    }

    let mut prev_digest: Option<String> = None;
    let mut prev_state_hash: Option<String> = None;
    let mut expected_index: Option<u64> = None;

    for (_i, wire) in receipts.into_iter().enumerate() {
        let step_index = wire.step_index;
        
        // 1. Isolation check
        let res = verify_micro(wire.clone());
        if res.decision == Decision::Reject {
            return VerifyChainResult {
                decision: Decision::Reject,
                code: res.code,
                failing_step: Some(step_index),
            };
        }

        // 2. Continuity checks
        if let Some(prev) = prev_digest {
            if wire.chain_digest_prev != prev {
                return VerifyChainResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectChainDigestPrev),
                    failing_step: Some(step_index),
                };
            }
        }
        if let Some(prev) = prev_state_hash {
            if wire.state_hash_prev != prev {
                return VerifyChainResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectStateHashLink),
                    failing_step: Some(step_index),
                };
            }
        }
        if let Some(expected) = expected_index {
            if wire.step_index != expected {
                 // Technically we don't have a specific reject code for index gap in the enum, 
                 // but we can use RejectSchema or a generic one. Let's assume RejectSchema for now
                 // or maybe RejectVersion if we want to be picky. 
                 // Actually, let's use RejectSchema for structural gaps.
                return VerifyChainResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectSchema),
                    failing_step: Some(step_index),
                };
            }
        }

        prev_digest = Some(wire.chain_digest_next);
        prev_state_hash = Some(wire.state_hash_next);
        expected_index = Some(wire.step_index + 1);
    }

    VerifyChainResult {
        decision: Decision::Accept,
        code: None,
        failing_step: None,
    }
}
