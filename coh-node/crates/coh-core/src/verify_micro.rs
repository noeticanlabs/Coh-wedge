use crate::auth::{verify_signature, VerifierContext};
use crate::canon::{
    to_canonical_json_bytes, to_prehash_view, EXPECTED_CANON_PROFILE_HASH,
    EXPECTED_MICRO_SCHEMA_ID, EXPECTED_MICRO_VERSION,
};
use crate::hash::compute_chain_digest;
use crate::math::CheckedMath;
use crate::semantic::SemanticRegistry;
use crate::types::{
    AdmissionProfile, Decision, MicroReceipt, MicroReceiptWire, RejectCode, VerifyMicroResult,
};
use std::convert::TryFrom;

/// Production verifier entrypoint that requires a trusted authority context.
/// This is the primary entrypoint for signature verification enforcement.
#[must_use]
pub fn verify_micro_with_context(
    wire: MicroReceiptWire,
    ctx: VerifierContext,
) -> VerifyMicroResult {
    let step_index = wire.step_index;
    let object_id = wire.object_id.clone();

    // 1. Wire to runtime (handles hex validation and numeric parsing)
    let r = match MicroReceipt::try_from(wire) {
        Ok(r) => r,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Malformed input: invalid hex or numeric format".to_string(),
                step_index: Some(step_index),
                object_id: Some(object_id),
                chain_digest_next: None,
            }
        }
    };

    // 2. Schema check
    if r.schema_id != EXPECTED_MICRO_SCHEMA_ID {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Invalid schema_id: {} (Expected: {})",
                r.schema_id, EXPECTED_MICRO_SCHEMA_ID
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }
    if r.version != EXPECTED_MICRO_VERSION {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectSchema),
            message: format!(
                "Unsupported version: {} (Expected: {})",
                r.version, EXPECTED_MICRO_VERSION
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 3. Object ID sanity
    if r.object_id.trim().is_empty() {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectMissingObjectId),
            message: "Missing object_id".to_string(),
            step_index: Some(r.step_index),
            object_id: Some("".to_string()),
            chain_digest_next: None,
        };
    }

    // 3.5 Signature Presence Check
    let missing_sig = match &r.signatures {
        None => true,
        Some(sigs) => sigs.is_empty(),
    };

    if missing_sig {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectMissingSignature),
            message: "Missing required signature(s)".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 4. Signature AUTHENTICATION - ENFORCED
    // This is the critical security check that verifies the signature is valid
    if let Some(sigs) = &r.signatures {
        for sig in sigs {
            if let Err(e) = verify_signature(&r, sig, None, None, &ctx) {
                return VerifyMicroResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!("Signature verification failed: {:?}", e),
                    step_index: Some(r.step_index),
                    object_id: Some(r.object_id),
                    chain_digest_next: None,
                };
            }
        }
    }

    // 5. Profile check
    if r.canon_profile_hash.to_hex() != EXPECTED_CANON_PROFILE_HASH {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectCanonProfile),
            message: format!(
                "Canonical profile mismatch: {} (Expected: {})",
                r.canon_profile_hash.to_hex(),
                EXPECTED_CANON_PROFILE_HASH
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 5. Policy logic (Arithmetic boundary check)
    // Constraint: v_post + spend <= v_pre + defect
    let lhs = match r.metrics.v_post.safe_add(r.metrics.spend) {
        Ok(val) => val,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Policy arithmetic overflow (v_post + spend)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: None,
            }
        }
    };
    let rhs = match r.metrics.v_pre.safe_add(r.metrics.defect) {
        Ok(val) => match val.safe_add(r.metrics.authority) {
            Ok(v) => v,
            Err(e) => {
                return VerifyMicroResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: "Policy arithmetic overflow (v_pre + defect + authority)".to_string(),
                    step_index: Some(r.step_index),
                    object_id: Some(r.object_id),
                    chain_digest_next: None,
                }
            }
        },
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Policy arithmetic overflow (v_pre + defect)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: None,
            }
        }
    };

    if lhs > rhs {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectPolicyViolation),
            message: format!(
                "Policy violation: v_post + spend ({}) exceeds v_pre + defect + authority ({})",
                lhs, rhs
            ),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 5. Semantic integrity checks (TypeConfusion defense)
    // C1: No vacuous zero receipts
    if r.metrics.v_pre == 0 && r.metrics.v_post == 0 && r.metrics.spend == 0 && r.metrics.defect == 0 {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::VacuousZeroReceipt),
            message: "Vacuous zero receipt: all coherence metrics are zero".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // C4: Cannot spend more than balance (spend <= v_pre)
    if r.metrics.spend > r.metrics.v_pre {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::SpendExceedsBalance),
            message: format!("Spend ({}) > v_pre ({})", r.metrics.spend, r.metrics.v_pre),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: None,
        };
    }

    // 6. Cryptographic integrity
    let prehash = to_prehash_view(&r);
    let canon_bytes = match to_canonical_json_bytes(&prehash) {
        Ok(bytes) => bytes,
        Err(e) => {
            return VerifyMicroResult {
                decision: Decision::Reject,
                code: Some(e),
                message: "Canonicalization failed".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: None,
            };
        }
    };
    let computed_digest = compute_chain_digest(r.chain_digest_prev, &canon_bytes);
    if computed_digest != r.chain_digest_next {
        return VerifyMicroResult {
            decision: Decision::Reject,
            code: Some(RejectCode::RejectChainDigest),
            message: "Cryptographic digest mismatch".to_string(),
            step_index: Some(r.step_index),
            object_id: Some(r.object_id),
            chain_digest_next: Some(r.chain_digest_next.to_hex()),
        };
    }

    // 7. Profile-specific admission
    match r.profile {
        AdmissionProfile::CoherenceOnlyV1 => {
            // Already checked by coherence resource law above
            VerifyMicroResult {
                decision: Decision::Accept,
                code: None,
                message: "Verified successfully (CoherenceOnlyV1)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: Some(r.chain_digest_next.to_hex()),
            }
        }
        AdmissionProfile::FormationV2 => {
            // 7.1 Law of Chaos check (Checked arithmetic)
            let chaos_lhs = r.metrics.m_post.checked_add(r.metrics.c_cost);
            let chaos_rhs = r.metrics.m_pre.checked_add(r.metrics.d_slack);

            match (chaos_lhs, chaos_rhs) {
                (Some(lhs), Some(rhs)) if lhs <= rhs => {
                    // OK
                }
                (Some(lhs), Some(rhs)) => {
                    return VerifyMicroResult {
                        decision: Decision::Reject,
                        code: Some(RejectCode::ChaosViolation),
                        message: format!("Chaos violation: M(g') + C(p) ({}) > M(g) + D(p) ({})", lhs, rhs),
                        step_index: Some(r.step_index),
                        object_id: Some(r.object_id),
                        chain_digest_next: None,
                    };
                }
                _ => {
                    return VerifyMicroResult {
                        decision: Decision::Reject,
                        code: Some(RejectCode::RejectOverflow),
                        message: "Chaos arithmetic overflow".to_string(),
                        step_index: Some(r.step_index),
                        object_id: Some(r.object_id),
                        chain_digest_next: None,
                    };
                }
            }

            // 7.2 Projection Link Verification: Pi(z) = (x, R, y)
            // The projection hash MUST match the deterministic hash of the coherence transition.
            let expected_projection = compute_projection_hash(&r);
            if r.metrics.projection_hash != expected_projection {
                return VerifyMicroResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::ProjectionMismatch),
                    message: format!(
                        "Projection link violation: metrics.projection_hash mismatch. Expected: {}",
                        expected_projection.to_hex()
                    ),
                    step_index: Some(r.step_index),
                    object_id: Some(r.object_id),
                    chain_digest_next: None,
                };
            }

            // 7.3 Semantic Envelope Check (delta_hat <= defect)
            if let Err(e) = SemanticRegistry::verify_defect_bound(&r) {
                let delta_hat_str = SemanticRegistry::delta_hat(&r.step_type)
                    .map(|(d, _)| d.to_string())
                    .unwrap_or_else(|_| "UNKNOWN".to_string());
                
                return VerifyMicroResult {
                    decision: Decision::Reject,
                    code: Some(e),
                    message: format!(
                        "Semantic envelope violation: defect ({}) < delta_hat ({})",
                        r.metrics.defect,
                        delta_hat_str
                    ),
                    step_index: Some(r.step_index),
                    object_id: Some(r.object_id),
                    chain_digest_next: None,
                };
            }

            // 7.4 Identity Constraint: spend must be zero for identity steps
            if SemanticRegistry::is_identity(&r.step_type) && r.metrics.spend != 0 {
                return VerifyMicroResult {
                    decision: Decision::Reject,
                    code: Some(RejectCode::SemanticEnvelopeViolation),
                    message: "Identity step cannot have non-zero spend".to_string(),
                    step_index: Some(r.step_index),
                    object_id: Some(r.object_id),
                    chain_digest_next: None,
                };
            }

            VerifyMicroResult {
                decision: Decision::Accept,
                code: None,
                message: "Verified successfully (FormationV2)".to_string(),
                step_index: Some(r.step_index),
                object_id: Some(r.object_id),
                chain_digest_next: Some(r.chain_digest_next.to_hex()),
            }
        }
    }
}

/// Compute the deterministic projection hash for a receipt's coherence transition.
/// This binds the Chaos generation layer to the specific executable claim.
pub fn compute_projection_hash(r: &MicroReceipt) -> crate::types::Hash32 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(r.object_id.as_bytes());
    hasher.update(&r.step_index.to_be_bytes());
    hasher.update(&r.metrics.v_pre.to_be_bytes());
    hasher.update(&r.metrics.v_post.to_be_bytes());
    hasher.update(&r.metrics.spend.to_be_bytes());
    hasher.update(&r.metrics.defect.to_be_bytes());
    hasher.update(&r.metrics.authority.to_be_bytes());
    hasher.update(&r.state_hash_prev.0);
    hasher.update(&r.state_hash_next.0);
    crate::types::Hash32(hasher.finalize().into())
}

/// Legacy verifier entrypoint that uses default fixture context for backward compatibility.
/// NOTE: Signature verification IS now enforced with the default context.
#[must_use]
pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult {
    // Use default fixture context - signature verification is now enforced
    let ctx = VerifierContext::fixture_default();
    verify_micro_with_context(wire, ctx)
}
