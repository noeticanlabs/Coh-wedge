import Mathlib.Data.ByteArray
import Coh.Contract.Slab -- reuse SlabSummary, policy/overflow checks, and Merkle axiom
import Coh.Contract.Boundary
import Coh.Contract.RejectCode

/-!
# Coh.Contract.SlabV2

ByteArray-digest canonical slab receipt surface. Mirrors `Coh.Contract.Slab`
but upgrades state-hash and chain-digest fields to `ByteArray`-based `Digest`.
This file provides structural predicates and a Decision-returning structural
verifier, plus view-style lemmas exposing the conjunction shape.
-/

namespace Coh.Contract

open Coh.Contract

/-- Canonical slab receipt with ByteArray digests for chain and state fields. -/
structure SlabReceiptV2 where
  schemaId         : SchemaId
  version          : String
  objectId         : String
  canonProfileHash : String
  policyHash       : String
  rangeStart       : Nat
  rangeEnd         : Nat
  microCount       : Nat
  chainDigestPrev  : Digest
  chainDigestNext  : Digest
  stateHashFirst   : Digest
  stateHashLast    : Digest
  /-- We keep Merkle root as String to reuse the existing Merkle axiom surface. -/
  merkleRoot       : String
  summary          : SlabSummary
  /-- Canonical bytes of the summary payload used in chain update. -/
  canonicalBytes   : ByteArray
  deriving Repr, DecidableEq

/-- Structural schema check for Slab V2. -/
def slabV2SchemaOk (cfg : ContractConfig) (r : SlabReceiptV2) : Prop :=
  r.schemaId = cfg.slabSchema ∧ r.version = cfg.slabVersion

def slabV2Nonempty (r : SlabReceiptV2) : Prop := 0 < r.microCount
def slabV2RangeValid (r : SlabReceiptV2) : Prop := r.rangeStart ≤ r.rangeEnd
def slabV2RangeCountMatches (r : SlabReceiptV2) : Prop :=
  r.microCount = r.rangeEnd - r.rangeStart + 1

def slabV2SummaryNoOverflow (r : SlabReceiptV2) : Prop :=
  r.summary.vPostLast + r.summary.totalSpend ≤ u128Max ∧
  r.summary.vPreFirst + r.summary.totalDefect ≤ u128Max

def slabV2SummaryPolicyLawful (r : SlabReceiptV2) : Prop :=
  r.summary.vPostLast + r.summary.totalSpend ≤ r.summary.vPreFirst + r.summary.totalDefect

/-- State span linkage at the slab boundary. -/
def slabV2StateSpanOk (first last : Digest) (r : SlabReceiptV2) : Prop :=
  r.stateHashFirst = first ∧ r.stateHashLast = last

/-- Chain binding for the slab summary payload. -/
def slabV2ChainOk (cp : CanonProfile) (prev : Digest) (r : SlabReceiptV2) : Prop :=
  r.chainDigestPrev = prev ∧ r.chainDigestNext = chainUpdate cp prev r.canonicalBytes

/-- Envelope-level structural conjunction. -/
def slabV2EnvelopePred (cfg : ContractConfig) (r : SlabReceiptV2) : Prop :=
  slabV2SchemaOk cfg r ∧
  slabV2Nonempty r ∧
  slabV2RangeValid r ∧
  slabV2RangeCountMatches r ∧
  slabV2SummaryNoOverflow r ∧
  slabV2SummaryPolicyLawful r

/-- Full structural conjunction (envelope + state + chain). -/
def slabV2StructPred (cfg : ContractConfig) (cp : CanonProfile)
    (first last prev : Digest) (r : SlabReceiptV2) : Prop :=
  slabV2EnvelopePred cfg r ∧ slabV2StateSpanOk first last r ∧ slabV2ChainOk cp prev r

/-- Structural verifier returning Decision RejectCode (without Merkle). -/
def verifySlabStructV2 (cfg : ContractConfig) (cp : CanonProfile)
    (first last prev : Digest) (r : SlabReceiptV2) : Decision RejectCode := by
  classical
  if hEnv : slabV2EnvelopePred cfg r then
    if hState : slabV2StateSpanOk first last r then
      if hChain : slabV2ChainOk cp prev r then
        exact Decision.accept
      else
        exact Decision.reject RejectCode.rejectChainDigest
    else
      exact Decision.reject RejectCode.rejectSlabSummary
  else
    exact Decision.reject RejectCode.rejectSchema

/-- Envelope synonym lemma. -/
@[simp]
theorem slabV2EnvelopePred_iff (cfg : ContractConfig) (r : SlabReceiptV2) :
    slabV2EnvelopePred cfg r ↔
      slabV2SchemaOk cfg r ∧ slabV2Nonempty r ∧ slabV2RangeValid r ∧
      slabV2RangeCountMatches r ∧ slabV2SummaryNoOverflow r ∧ slabV2SummaryPolicyLawful r := Iff.rfl

namespace slabV2StructPred

variable {cfg : ContractConfig} {cp : CanonProfile}
variable {first last prev : Digest} {r : SlabReceiptV2}

theorem envelope (h : slabV2StructPred cfg cp first last prev r) :
    slabV2EnvelopePred cfg r := h.left

theorem stateSpan (h : slabV2StructPred cfg cp first last prev r) :
    slabV2StateSpanOk first last r := (h.right).left

theorem chain (h : slabV2StructPred cfg cp first last prev r) :
    slabV2ChainOk cp prev r := (h.right).right

end slabV2StructPred

end Coh.Contract
