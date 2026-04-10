use crate::types::{Decision, MicroReceipt, MicroReceiptWire, RejectCode, VerifyChainResult};
use crate::verify_micro::verify_micro;
use std::convert::TryFrom;

#[must_use]
pub fn verify_chain(receipts: Vec<MicroReceiptWire>) -> VerifyChainResult {
    if receipts.is_empty() {
        return VerifyChainResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: "Empty chain provided: proof requires at least one micro-receipt.".to_string(),
            steps_verified: 0,
            first_step_index: 0,
            last_step_index: 0,
            final_chain_digest: None,
            failing_step_index: None,
            steps_verified_before_failure: Some(0),
        };
    }

    let mut first_index = 0;
    let mut last_good_index = 0;
    let mut current_digest: Option<String> = None;
    let mut current_state: Option<String> = None;

    for (i, wire) in receipts.into_iter().enumerate() {
        let step_idx = wire.step_index;
        if i == 0 {
            first_index = step_idx;
        }

        // 1. Verify in isolation
        let res = verify_micro(wire.clone());
        if res.decision == Decision::Reject {
            return VerifyChainResult {
                decision: Decision::Reject,
                code: res.code,
                message: format!("Semantic rejection at step {}: {}", step_idx, res.message),
                steps_verified: i as u64,
                first_step_index: first_index,
                last_step_index: if i == 0 { first_index } else { last_good_index },
                final_chain_digest: current_digest,
                failing_step_index: Some(step_idx),
                steps_verified_before_failure: Some(i as u64),
            };
        }

        let r = MicroReceipt::try_from(wire).unwrap();

        // 2. Continuity checks
        if i > 0 {
            // Index continuity check: must be strictly +1
            if r.step_index != last_good_index + 1 {
                return VerifyChainResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::RejectSchema),
                    message: format!("Index discontinuity at step {}: expected next step to be {}, but found {}.", step_idx, last_good_index + 1, r.step_index),
                    steps_verified: i as u64,
                    first_step_index: first_index,
                    last_step_index: last_good_index,
                    final_chain_digest: current_digest,
                    failing_step_index: Some(step_idx),
                    steps_verified_before_failure: Some(i as u64),
                };
            }

            // Check link to previous
            if let Some(ref prev_digest) = current_digest {
                if r.chain_digest_prev.to_hex() != *prev_digest {
                    return VerifyChainResult {
                        decision: Decision::Reject,
                        code: Some(RejectCode::RejectChainDigest),
                        message: format!("Chain digest link broken at step {}: expected link to {}, but found {}.", step_idx, prev_digest, r.chain_digest_prev.to_hex()),
                        steps_verified: i as u64,
                        first_step_index: first_index,
                        last_step_index: last_good_index,
                        final_chain_digest: Some(prev_digest.clone()),
                        failing_step_index: Some(step_idx),
                        steps_verified_before_failure: Some(i as u64),
                    };
                }
            }
            if let Some(ref prev_state) = current_state {
                if r.state_hash_prev.to_hex() != *prev_state {
                    return VerifyChainResult {
                        decision: Decision::Reject,
                        code: Some(RejectCode::RejectStateHashLink),
                        message: format!(
                            "State link broken at step {}: expected link to {}, but found {}.",
                            step_idx,
                            prev_state,
                            r.state_hash_prev.to_hex()
                        ),
                        steps_verified: i as u64,
                        first_step_index: first_index,
                        last_step_index: last_good_index,
                        final_chain_digest: current_digest,
                        failing_step_index: Some(step_idx),
                        steps_verified_before_failure: Some(i as u64),
                    };
                }
            }
        }

        last_good_index = step_idx;
        current_digest = Some(r.chain_digest_next.to_hex());
        current_state = Some(r.state_hash_next.to_hex());
    }

    VerifyChainResult {
        decision: Decision::Accept,
        code: None,
        message: format!(
            "Linear proof chain verified: {} contiguous steps accepted.",
            last_good_index - first_index + 1
        ),
        steps_verified: (last_good_index - first_index + 1),
        first_step_index: first_index,
        last_step_index: last_good_index,
        final_chain_digest: current_digest,
        failing_step_index: None,
        steps_verified_before_failure: None,
    }
}
