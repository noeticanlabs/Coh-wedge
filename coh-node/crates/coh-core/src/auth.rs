use crate::canon::to_canonical_json_bytes;
use crate::fixtures::finalize_micro_receipt;
use crate::reject::RejectCode;
use crate::types::{MicroReceipt, MicroReceiptWire, SignatureWire};
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

pub const COHENC_V1_SIGNED_TRANSITION_TAG: &[u8] = b"COHENC_V1_SIGNED_TRANSITION";
pub const DEFAULT_SCOPE: &str = "*";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SignedTransitionPayload {
    pub authority_id: String,
    pub object_id: String,
    pub policy_hash: String,
    pub prev_chain_digest: String,
    pub receipt_type: String,
    pub scope: String,
    pub state_hash_post: String,
    pub state_hash_pre: String,
    pub step: u64,
    pub v_post: String,
    pub v_pre: String,
    pub spend: String,
    pub defect: String,
    pub authority: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScopePolicy {
    pub allowed_scopes: Vec<String>,
    pub object_prefixes: Vec<String>,
}

impl ScopePolicy {
    pub fn allow_all() -> Self {
        Self {
            allowed_scopes: vec![DEFAULT_SCOPE.to_string()],
            object_prefixes: vec![DEFAULT_SCOPE.to_string()],
        }
    }

    pub fn allows(&self, scope: &str, object_id: &str) -> bool {
        let scope_ok = self.allowed_scopes.is_empty()
            || self
                .allowed_scopes
                .iter()
                .any(|allowed| allowed == DEFAULT_SCOPE || allowed == scope);
        let object_ok = self.object_prefixes.is_empty()
            || self
                .object_prefixes
                .iter()
                .any(|prefix| prefix == DEFAULT_SCOPE || object_id.starts_with(prefix));
        scope_ok && object_ok
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedAuthority {
    pub authority_id: String,
    pub public_key: String,
    pub trusted: bool,
    pub scope_policy: ScopePolicy,
    pub expires_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerifierContext {
    pub trusted_signers: BTreeMap<String, TrustedAuthority>,
    pub active_policy_hash: Option<String>,
    pub current_time: Option<u64>,
}

impl VerifierContext {
    pub fn fixture_default() -> Self {
        let mut trusted_signers = BTreeMap::new();
        for authority_id in [
            "fixture-signer-0",
            "fixture-signer-1",
            "fixture-signer-2",
            "test_signer",
            "coh-sidecar",
        ] {
            let key = fixture_signing_key(authority_id);
            trusted_signers.insert(
                authority_id.to_string(),
                TrustedAuthority {
                    authority_id: authority_id.to_string(),
                    public_key: hex::encode(key.verifying_key().to_bytes()),
                    trusted: true,
                    scope_policy: ScopePolicy::allow_all(),
                    expires_at: None,
                },
            );
        }

        Self {
            trusted_signers,
            // Empty string means no policy hash enforcement (for backward compatibility)
            active_policy_hash: Some(String::new()),
            current_time: None,
        }
    }
}

pub fn canonical_signed_transition_bytes(
    receipt: &MicroReceipt,
    authority_id: &str,
    scope: &str,
    receipt_type: &str,
) -> Result<Vec<u8>, RejectCode> {
    let payload = SignedTransitionPayload {
        authority_id: authority_id.to_string(),
        object_id: receipt.object_id.clone(),
        policy_hash: receipt.policy_hash.to_hex(),
        prev_chain_digest: receipt.chain_digest_prev.to_hex(),
        receipt_type: receipt_type.to_string(),
        scope: scope.to_string(),
        state_hash_post: receipt.state_hash_next.to_hex(),
        state_hash_pre: receipt.state_hash_prev.to_hex(),
        step: receipt.step_index,
        v_post: receipt.metrics.v_post.to_string(),
        v_pre: receipt.metrics.v_pre.to_string(),
        spend: receipt.metrics.spend.to_string(),
        defect: receipt.metrics.defect.to_string(),
        authority: receipt.metrics.authority.to_string(),
        version: receipt.version.clone(),
    };

    let canonical = to_canonical_json_bytes(&payload)?;
    let mut out = Vec::with_capacity(COHENC_V1_SIGNED_TRANSITION_TAG.len() + 1 + canonical.len());
    out.extend_from_slice(COHENC_V1_SIGNED_TRANSITION_TAG);
    out.extend_from_slice(b"|");
    out.extend_from_slice(&canonical);
    Ok(out)
}

pub fn sign_micro_receipt(
    wire: MicroReceiptWire,
    signing_key: &SigningKey,
    authority_id: &str,
    scope: &str,
    timestamp: u64,
    expires_at: Option<u64>,
    receipt_type: &str,
) -> Result<MicroReceiptWire, RejectCode> {
    let mut finalized = finalize_micro_receipt(wire)?;
    let runtime = MicroReceipt::try_from(finalized.clone())?;
    let signed_bytes =
        canonical_signed_transition_bytes(&runtime, authority_id, scope, receipt_type)?;
    let signature = signing_key.sign(&signed_bytes);
    finalized.signatures = Some(vec![SignatureWire {
        signature: base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
        signer: authority_id.to_string(),
        timestamp,
        authority_id: Some(authority_id.to_string()),
        scope: Some(scope.to_string()),
        expires_at,
    }]);
    Ok(finalized)
}

pub fn fixture_signing_key(authority_id: &str) -> SigningKey {
    let seed = match authority_id {
        "fixture-signer-0" => [7u8; 32],
        "fixture-signer-1" => [8u8; 32],
        "fixture-signer-2" => [9u8; 32],
        "test_signer" => [10u8; 32],
        "coh-sidecar" => [11u8; 32],
        _ => [12u8; 32],
    };
    SigningKey::from_bytes(&seed)
}

pub fn decode_verifying_key(encoded: &str) -> Result<VerifyingKey, RejectCode> {
    let bytes = hex::decode(encoded)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(encoded))
        .map_err(|_| RejectCode::RejectSignatureMalformed)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| RejectCode::RejectSignatureMalformed)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| RejectCode::RejectSignatureMalformed)
}

pub fn decode_signature(encoded: &str) -> Result<Signature, RejectCode> {
    let bytes = hex::decode(encoded)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(encoded))
        .map_err(|_| RejectCode::RejectSignatureMalformed)?;
    let arr: [u8; 64] = bytes
        .try_into()
        .map_err(|_| RejectCode::RejectSignatureMalformed)?;
    Ok(Signature::from_bytes(&arr))
}

/// Verify a single signature against a receipt using the given verifier context.
/// Returns Ok(()) on success, or Err(RejectCode) on failure.
pub fn verify_signature(
    receipt: &MicroReceipt,
    sig: &SignatureWire,
    authority_id_opt: Option<&str>,
    scope_opt: Option<&str>,
    ctx: &VerifierContext,
) -> Result<(), RejectCode> {
    // 1. Find the authority_id from the signature or use provided
    let authority_id = authority_id_opt
        .or(sig.authority_id.as_deref())
        .unwrap_or(&sig.signer);

    // 2. Look up trusted authority
    let authority = ctx
        .trusted_signers
        .get(authority_id)
        .ok_or(RejectCode::RejectSignerUnknown)?;

    // 3. Verify the signer is trusted
    if !authority.trusted {
        return Err(RejectCode::RejectSignerUntrusted);
    }

    // 4. Check expiration if configured
    if let Some(current_time) = ctx.current_time {
        if let Some(expires_at) = sig.expires_at {
            if current_time > expires_at {
                return Err(RejectCode::RejectSignatureExpired);
            }
        }
        if let Some(expires_at) = authority.expires_at {
            if current_time > expires_at {
                return Err(RejectCode::RejectSignatureExpired);
            }
        }
    }

    // 5. Check scope policy
    let scope = scope_opt.or(sig.scope.as_deref()).unwrap_or(DEFAULT_SCOPE);
    if !authority.scope_policy.allows(scope, &receipt.object_id) {
        return Err(RejectCode::RejectSignatureScopeMismatch);
    }

    // 6. Check policy hash match if active policy is enforced
    if let Some(active_policy_hash) = &ctx.active_policy_hash {
        if !active_policy_hash.is_empty() && active_policy_hash != &receipt.policy_hash.to_hex() {
            return Err(RejectCode::RejectSignaturePolicyMismatch);
        }
    }

    // 7. Decode the public key and signature
    let public_key = decode_verifying_key(&authority.public_key)?;
    let signature = decode_signature(&sig.signature)?;

    // 8. Compute the canonical signed transition bytes
    let signed_bytes =
        canonical_signed_transition_bytes(receipt, authority_id, scope, "MICRO_RECEIPT_V1")?;

    // 9. Perform Ed25519 verification
    public_key
        .verify_strict(&signed_bytes, &signature)
        .map_err(|_| RejectCode::RejectSignatureBad)
}
