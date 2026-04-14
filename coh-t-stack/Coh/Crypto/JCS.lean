import Coh.Crypto.Bytes
import Coh.Contract.Micro

namespace Coh.Crypto

open Coh.Core
open Coh.Contract

/-- Minimal quoted-string renderer for the frozen receipt fragment. -/
def jcsQuote (s : String) : ByteSeq :=
  s!"\"{s}\""

/-- Receipt projection matching the Rust `to_prehash_view` payload. -/
structure ReceiptProjection where
  canonProfileHash : CanonProfileHash
  chainDigestPrev : ChainDigest
  metrics : Metrics
  objectId : String
  policyHash : PolicyHash
  schemaId : SchemaId
  stateHashNext : StateHash
  stateHashPrev : StateHash
  stepIndex : Nat
  version : String
  deriving Repr, DecidableEq

/-- Lean-side projection to the exact hashed receipt fragment. -/
def receiptProjectionOf (r : MicroReceipt) : ReceiptProjection :=
  { canonProfileHash := r.canonProfileHash
    chainDigestPrev := r.chainDigestPrev
    metrics := r.metrics
    objectId := r.objectId
    policyHash := r.policyHash
    schemaId := r.schemaId
    stateHashNext := r.stateHashNext
    stateHashPrev := r.stateHashPrev
    stepIndex := r.stepIndex
    version := r.version }

/-- Canonical metrics JSON matching the Rust `MetricsPrehash` field order. -/
def canonicalMetricsJson (m : Metrics) : ByteSeq :=
  "{" ++
    jcsQuote "defect" ++ ":" ++ jcsQuote (toString m.defect) ++ "," ++
    jcsQuote "spend" ++ ":" ++ jcsQuote (toString m.spend) ++ "," ++
    jcsQuote "v_post" ++ ":" ++ jcsQuote (toString m.vPost) ++ "," ++
    jcsQuote "v_pre" ++ ":" ++ jcsQuote (toString m.vPre) ++
  "}"

/-- Canonical JSON bytes for the projected Rust prehash surface. -/
def receiptProjectionCanonicalJson (p : ReceiptProjection) : ByteSeq :=
  "{" ++
    jcsQuote "canon_profile_hash" ++ ":" ++ jcsQuote p.canonProfileHash ++ "," ++
    jcsQuote "chain_digest_prev" ++ ":" ++ jcsQuote p.chainDigestPrev.bytes ++ "," ++
    jcsQuote "metrics" ++ ":" ++ canonicalMetricsJson p.metrics ++ "," ++
    jcsQuote "object_id" ++ ":" ++ jcsQuote p.objectId ++ "," ++
    jcsQuote "policy_hash" ++ ":" ++ jcsQuote p.policyHash ++ "," ++
    jcsQuote "schema_id" ++ ":" ++ jcsQuote p.schemaId ++ "," ++
    jcsQuote "state_hash_next" ++ ":" ++ jcsQuote p.stateHashNext.value ++ "," ++
    jcsQuote "state_hash_prev" ++ ":" ++ jcsQuote p.stateHashPrev.value ++ "," ++
    jcsQuote "step_index" ++ ":" ++ toString p.stepIndex ++ "," ++
    jcsQuote "version" ++ ":" ++ jcsQuote p.version ++
  "}"

/-- Canonical receipt JSON bytes for the frozen Coh receipt fragment. -/
def canonicalMicroJson (r : MicroReceipt) : ByteSeq :=
  receiptProjectionCanonicalJson (receiptProjectionOf r)

theorem canonicalMetricsJson_matches_rust_field_order (m : Metrics) :
    canonicalMetricsJson m =
      "{" ++
        jcsQuote "defect" ++ ":" ++ jcsQuote (toString m.defect) ++ "," ++
        jcsQuote "spend" ++ ":" ++ jcsQuote (toString m.spend) ++ "," ++
        jcsQuote "v_post" ++ ":" ++ jcsQuote (toString m.vPost) ++ "," ++
        jcsQuote "v_pre" ++ ":" ++ jcsQuote (toString m.vPre) ++
      "}" := rfl

theorem receipt_projection_eq (r : MicroReceipt) :
    receiptProjectionOf r =
      { canonProfileHash := r.canonProfileHash
        chainDigestPrev := r.chainDigestPrev
        metrics := r.metrics
        objectId := r.objectId
        policyHash := r.policyHash
        schemaId := r.schemaId
        stateHashNext := r.stateHashNext
        stateHashPrev := r.stateHashPrev
        stepIndex := r.stepIndex
        version := r.version } := rfl

theorem JCS_bytes_eq (r : MicroReceipt) :
    canonicalMicroJson r = receiptProjectionCanonicalJson (receiptProjectionOf r) := rfl

end Coh.Crypto
