import Coh.Crypto.HashBridge

namespace Coh.Contract

open Coh.Core
open Coh.Crypto

/-- Re-export the canonical types from the crypto layer for contract-level use. -/
abbrev jcsQuote := Coh.Crypto.jcsQuote
abbrev ReceiptProjection := Coh.Crypto.ReceiptProjection
abbrev receiptProjectionOf := Coh.Crypto.receiptProjectionOf
abbrev canonicalMetricsJson := Coh.Crypto.canonicalMetricsJson
abbrev receiptProjectionCanonicalJson := Coh.Crypto.receiptProjectionCanonicalJson
abbrev canonicalMicroJson := Coh.Crypto.canonicalMicroJson
abbrev rustChainDigestInputBytes := Coh.Crypto.rustChainDigestInputBytes
abbrev canonicalDigestInputBytes := Coh.Crypto.canonicalDigestInputBytes
abbrev PayloadMatchesCanonicalJson := Coh.Crypto.PayloadMatchesCanonicalJson

/-- Canonical metrics JSON matcher. -/
theorem canonicalMetricsJson_matches_rust_field_order (m : Metrics) :
    canonicalMetricsJson m = Coh.Crypto.canonicalMetricsJson m := rfl

/-- Receipt projection equality. -/
theorem receipt_projection_eq (r : MicroReceipt) :
    receiptProjectionOf r = Coh.Crypto.receiptProjectionOf r := rfl

/-- JCS canonical bytes equality. -/
theorem JCS_bytes_eq (r : MicroReceipt) :
    canonicalMicroJson r = Coh.Crypto.canonicalMicroJson r := rfl

/-- Chain preimage equality at the boundary. -/
theorem chain_preimage_eq (r : MicroReceipt) :
    canonicalDigestInputBytes r = Coh.Crypto.canonicalDigestInputBytes r := rfl

/-- Digest update refines the SHA-256 spec. -/
theorem digestUpdate_bytes_eq_canonicalDigestInputBytes
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev (canonicalize r).toString = hashBytes (canonicalDigestInputBytes r) :=
  Coh.Crypto.digestUpdate_refines_sha256_spec r hPayload

/-- Compute chain digest matches the spec. -/
theorem compute_chain_digest_eq_spec
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev (canonicalize r).toString =
      sha256_spec
        (rustChainDigestInputBytes r.chainDigestPrev
          (receiptProjectionCanonicalJson (receiptProjectionOf r))) :=
  Coh.Crypto.compute_chain_digest_eq_spec r hPayload

end Coh.Contract
