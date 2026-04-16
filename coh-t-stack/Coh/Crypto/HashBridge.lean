import Coh.Crypto.SHA256Spec
import Coh.Crypto.JCS

namespace Coh.Crypto

open Coh.Core
open Coh.Contract

/-!
## Important: Symbolic Hash Model

All theorems in this module prove *structural* refinement only (input-byte
layout correctness).  `sha256_spec` is a symbolic placeholder â€” see
`SHA256Spec.lean` for the full disclaimer.
-/
/-- Exact preimage bytes consumed by the Rust chain-digest function.
    This model directly mirrors the existing `digestPreimage` layout from the core hash module. -/
def rustChainDigestInputBytes (prevDigest : ChainDigest) (canonicalJsonBytes : ByteSeq) : ByteSeq :=
  s!"{digestDomainTag}|{prevDigest.bytes}|{canonicalJsonBytes}"

/-- Canonical digest input assembled from the projected receipt bytes. -/
def canonicalDigestInputBytes (r : MicroReceipt) : ByteSeq :=
  rustChainDigestInputBytes r.chainDigestPrev (canonicalMicroJson r)

/-- The receipt payload agrees with the canonical JCS bytes fed into hashing. -/
def PayloadMatchesCanonicalJson (r : MicroReceipt) : Prop :=
  r.canonicalPayload = canonicalMicroJson r

instance instDecidablePayloadMatchesCanonicalJson (r : MicroReceipt) :
    Decidable (PayloadMatchesCanonicalJson r) := by
  unfold PayloadMatchesCanonicalJson
  infer_instance

theorem rustChainDigestInputBytes_matches_hash_layout
    (prevDigest : ChainDigest) (canonicalJsonBytes : ByteSeq) :
    rustChainDigestInputBytes prevDigest canonicalJsonBytes =
      s!"{digestDomainTag}|{prevDigest.bytes}|{canonicalJsonBytes}" := rfl

theorem canonicalDigestInputBytes_matches_modeled_rust_layout (r : MicroReceipt) :
    canonicalDigestInputBytes r =
      s!"{digestDomainTag}|{r.chainDigestPrev.bytes}|{canonicalMicroJson r}" := rfl

theorem chain_preimage_eq (r : MicroReceipt) :
    canonicalDigestInputBytes r =
      rustChainDigestInputBytes r.chainDigestPrev
        (receiptProjectionCanonicalJson (receiptProjectionOf r)) := by
  simp [canonicalDigestInputBytes, JCS_bytes_eq]

theorem hashBytes_refines_sha256_spec_at_chain_boundary (input : ByteSeq) :
    hashBytes input = sha256_spec input := rfl

theorem digestUpdate_refines_sha256_spec
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev r.canonicalPayload = sha256_spec (canonicalDigestInputBytes r) := by
  simp [digestUpdate, sha256_spec, digestPreimage, canonicalDigestInputBytes, rustChainDigestInputBytes, hPayload]

theorem compute_chain_digest_eq_spec
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev r.canonicalPayload =
      sha256_spec
        (rustChainDigestInputBytes r.chainDigestPrev
          (receiptProjectionCanonicalJson (receiptProjectionOf r))) := by
  rw [digestUpdate_refines_sha256_spec r hPayload]
  rw [chain_preimage_eq r]

end Coh.Crypto

