import Mathlib.Data.ByteArray
import Coh.Contract.Schema
import Coh.Contract.Micro -- reuse Metrics and ContractConfig
import Coh.Contract.Boundary
import Coh.Contract.RejectCode

/-!
# Coh.Contract.MicroV2

ByteArray-digest canonical micro receipt surface. This module coexists with the
current `Coh.Contract.Micro` (string-backed hashes) to avoid churn while
freezing a canon-grade contract layer.
-/

namespace Coh.Contract

open Coh.Contract

/-- Canonical micro receipt with ByteArray digests. -/
structure MicroReceiptV2 where
  schemaId         : SchemaId
  version          : String
  objectId         : String
  canonProfileHash : String
  policyHash       : String
  stepIndex        : Nat
  stateHashPrev    : Digest
  stateHashNext    : Digest
  chainDigestPrev  : Digest
  chainDigestNext  : Digest
  canonicalBytes   : ByteArray
  metrics          : Metrics
  deriving Repr, DecidableEq

/-- Structural schema/policy/canon checks mirroring BoundaryConfig. -/
def microV2SchemaOk (bc : BoundaryConfig) (r : MicroReceiptV2) : Prop :=
  r.schemaId = bc.base.microSchema ∧
  r.version = bc.base.microVersion ∧
  r.objectId ≠ "" ∧
  r.canonProfileHash = bc.base.canonProfileHash ∧
  r.policyHash = bc.expectedPolicyHash

def microV2StateLinkOk (prevState nextState : Digest) (r : MicroReceiptV2) : Prop :=
  r.stateHashPrev = prevState ∧ r.stateHashNext = nextState

def microV2ChainOk (bc : BoundaryConfig) (prevChainDigest : Digest) (r : MicroReceiptV2) : Prop :=
  r.chainDigestPrev = prevChainDigest ∧
  r.chainDigestNext = chainUpdate bc.canonProfile prevChainDigest r.canonicalBytes

/-- Canonical structural verifier kernel for V2 micro receipts. -/
def verifyMicroStructV2
    (bc : BoundaryConfig)
    (prevState nextState prevChainDigest : Digest)
    (r : MicroReceiptV2) : Decision RejectCode := by
  classical
  if hSchema : microV2SchemaOk bc r then
    if hState : microV2StateLinkOk prevState nextState r then
      if hChain : microV2ChainOk bc prevChainDigest r then
        exact Decision.accept
      else
        exact Decision.reject RejectCode.rejectChainDigest
    else
      exact Decision.reject RejectCode.rejectStateHashLink
  else
    exact Decision.reject RejectCode.rejectSchema

end Coh.Contract
