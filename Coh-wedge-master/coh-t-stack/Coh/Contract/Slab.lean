import Coh.Contract.Micro

namespace Coh.Contract

open Coh.Core

structure SlabSummary where
  totalSpend : Nat
  totalDefect : Nat
  vPreFirst : Nat
  vPostLast : Nat
  deriving Repr, DecidableEq

structure SlabReceipt where
  schemaId : SchemaId
  version : String
  objectId : String
  canonProfileHash : CanonProfileHash
  policyHash : PolicyHash
  rangeStart : Nat
  rangeEnd : Nat
  microCount : Nat
  chainDigestPrev : ChainDigest
  chainDigestNext : ChainDigest
  stateHashFirst : StateHash
  stateHashLast : StateHash
  merkleRoot : String
  merkleWitnessValid : Bool
  summary : SlabSummary
  deriving Repr, DecidableEq

namespace SlabReceipt

def ValidSchema (cfg : ContractConfig) (r : SlabReceipt) : Prop :=
  r.schemaId = cfg.slabSchema ∧ r.version = cfg.slabVersion

instance instDecidableValidSchema (cfg : ContractConfig) (r : SlabReceipt) :
    Decidable (ValidSchema cfg r) := by
  unfold ValidSchema
  infer_instance

end SlabReceipt

/-!
## Merkle Boundary: Axiomatized Shim

Following the Coh "No-Bluff" Protocol, we replace the raw boolean oracle with
a thin axiomatized spec. This isolates the cryptographic assumption and
prepares for a future concrete Merkle implementation.

`MerkleInclusion` is the abstract proposition that an inclusion proof exists for
a given object in a Merkle tree with the specified root.

`merkle_oracle_consistent` is the founding axiom that the boolean flag populated
by the Rust environment is both sound and complete with respect to the
abstract Merkle specification.
-/

/-- Abstract specification for Merkle inclusion. -/
axiom MerkleInclusion (root : String) (objectId : String) : Prop

def MerklePathValid (r : SlabReceipt) : Prop :=
  r.merkleWitnessValid = true

/-- Foundational Axiom: The Oracle matches the Abstract Spec. -/
axiom merkle_oracle_consistent (r : SlabReceipt) :
  MerklePathValid r ↔ MerkleInclusion r.merkleRoot r.objectId

instance instDecidableMerklePathValid (r : SlabReceipt) : Decidable (MerklePathValid r) := by
  unfold MerklePathValid
  infer_instance

def NonemptySlab (r : SlabReceipt) : Prop :=
  0 < r.microCount

instance instDecidableNonemptySlab (r : SlabReceipt) : Decidable (NonemptySlab r) := by
  unfold NonemptySlab
  infer_instance

def RangeValid (r : SlabReceipt) : Prop :=
  r.rangeStart ≤ r.rangeEnd

instance instDecidableRangeValid (r : SlabReceipt) : Decidable (RangeValid r) := by
  unfold RangeValid
  infer_instance

def RangeCountMatches (r : SlabReceipt) : Prop :=
  r.microCount = r.rangeEnd - r.rangeStart + 1

instance instDecidableRangeCountMatches (r : SlabReceipt) :
    Decidable (RangeCountMatches r) := by
  unfold RangeCountMatches
  infer_instance

def SummaryNoOverflow (r : SlabReceipt) : Prop :=
  r.summary.vPostLast + r.summary.totalSpend ≤ u128Max ∧
    r.summary.vPreFirst + r.summary.totalDefect ≤ u128Max

instance instDecidableSummaryNoOverflow (r : SlabReceipt) :
    Decidable (SummaryNoOverflow r) := by
  unfold SummaryNoOverflow
  infer_instance

def SummaryPolicyLawful (r : SlabReceipt) : Prop :=
  r.summary.vPostLast + r.summary.totalSpend ≤ r.summary.vPreFirst + r.summary.totalDefect

instance instDecidableSummaryPolicyLawful (r : SlabReceipt) :
    Decidable (SummaryPolicyLawful r) := by
  unfold SummaryPolicyLawful
  infer_instance

def SummaryConsistent (r : SlabReceipt) : Prop :=
  NonemptySlab r ∧
    RangeValid r ∧
    RangeCountMatches r ∧
    SummaryNoOverflow r ∧
    SummaryPolicyLawful r

instance instDecidableSummaryConsistent (r : SlabReceipt) :
    Decidable (SummaryConsistent r) := by
  unfold SummaryConsistent
  infer_instance

def verifySlabEnvelope (cfg : ContractConfig) (r : SlabReceipt) : Bool :=
  decide (SlabReceipt.ValidSchema cfg r ∧ SummaryConsistent r)

def verifySlabWithMerkle (cfg : ContractConfig) (r : SlabReceipt) : Bool :=
  decide (SlabReceipt.ValidSchema cfg r ∧ SummaryConsistent r ∧ MerklePathValid r)

def verifySlabEnvelopeRejectCode (cfg : ContractConfig) (r : SlabReceipt) : Option RejectCode :=
  if ¬ SlabReceipt.ValidSchema cfg r then some RejectCode.rejectSchema
  else if ¬ NonemptySlab r then some RejectCode.rejectSlabSummary
  else if ¬ RangeValid r then some RejectCode.rejectSlabSummary
  else if ¬ RangeCountMatches r then some RejectCode.rejectSlabSummary
  else if ¬ SummaryNoOverflow r then some RejectCode.rejectOverflow
  else if ¬ SummaryPolicyLawful r then some RejectCode.rejectPolicyViolation
  else none

def verifySlabRejectCode (cfg : ContractConfig) (r : SlabReceipt) : Option RejectCode :=
  match verifySlabEnvelopeRejectCode cfg r with
  | some code => some code
  | none => if ¬ MerklePathValid r then some RejectCode.rejectSlabMerkle else none

def verifySlab (cfg : ContractConfig) (r : SlabReceipt) : Bool :=
  verifySlabWithMerkle cfg r

theorem verify_slab_envelope_accept_of_valid_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r) :
    verifySlabEnvelope cfg r = true := by
  unfold verifySlabEnvelope
  simp [hSchema, hSummary]

theorem verifySlabEnvelopeRejectCode_none_of_valid_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r) :
    verifySlabEnvelopeRejectCode cfg r = none := by
  rcases hSummary with ⟨hNonempty, hRange, hCount, hOverflow, hPolicy⟩
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hNonempty, hRange, hCount, hOverflow, hPolicy]

theorem verify_slab_accept_of_valid_merkle_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hMerkle : MerklePathValid r) :
    verifySlab cfg r = true := by
  unfold verifySlab verifySlabWithMerkle
  simp [hSchema, hSummary, hMerkle]

theorem verifySlabRejectCode_none_of_valid_merkle_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hMerkle : MerklePathValid r) :
    verifySlabRejectCode cfg r = none := by
  have hEnvelope : verifySlabEnvelopeRejectCode cfg r = none :=
    verifySlabEnvelopeRejectCode_none_of_valid_summary cfg r hSchema hSummary
  unfold verifySlabRejectCode
  simp [hEnvelope, hMerkle]

theorem verifySlabEnvelopeRejectCode_of_bad_schema
    (cfg : ContractConfig) (r : SlabReceipt)
    (hBadSchema : ¬ SlabReceipt.ValidSchema cfg r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectSchema := by
  unfold verifySlabEnvelopeRejectCode
  simp [hBadSchema]

theorem verifySlabEnvelopeRejectCode_of_empty
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hEmpty : ¬ NonemptySlab r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectSlabSummary := by
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hEmpty]

theorem verifySlabEnvelopeRejectCode_of_invalid_range
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hNonempty : NonemptySlab r)
    (hRange : ¬ RangeValid r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectSlabSummary := by
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hNonempty, hRange]

theorem verifySlabEnvelopeRejectCode_of_bad_count
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hNonempty : NonemptySlab r)
    (hRange : RangeValid r)
    (hCount : ¬ RangeCountMatches r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectSlabSummary := by
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hNonempty, hRange, hCount]

theorem verifySlabEnvelopeRejectCode_of_overflow
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hNonempty : NonemptySlab r)
    (hRange : RangeValid r)
    (hCount : RangeCountMatches r)
    (hOverflow : ¬ SummaryNoOverflow r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectOverflow := by
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hNonempty, hRange, hCount, hOverflow]

theorem verifySlabEnvelopeRejectCode_of_policy_violation
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hNonempty : NonemptySlab r)
    (hRange : RangeValid r)
    (hCount : RangeCountMatches r)
    (hNoOverflow : SummaryNoOverflow r)
    (hPolicy : ¬ SummaryPolicyLawful r) :
    verifySlabEnvelopeRejectCode cfg r = some RejectCode.rejectPolicyViolation := by
  unfold verifySlabEnvelopeRejectCode
  simp [hSchema, hNonempty, hRange, hCount, hNoOverflow, hPolicy]

theorem verifySlabRejectCode_of_bad_merkle
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hBadMerkle : ¬ MerklePathValid r) :
    verifySlabRejectCode cfg r = some RejectCode.rejectSlabMerkle := by
  have hEnvelope : verifySlabEnvelopeRejectCode cfg r = none :=
    verifySlabEnvelopeRejectCode_none_of_valid_summary cfg r hSchema hSummary
  unfold verifySlabRejectCode
  simp [hEnvelope, hBadMerkle]

theorem verify_slab_envelope_reject_of_empty
    (cfg : ContractConfig) (r : SlabReceipt)
    (hEmpty : ¬ NonemptySlab r) :
    verifySlabEnvelope cfg r = false := by
  unfold verifySlabEnvelope SummaryConsistent
  simp [hEmpty]

theorem verify_slab_envelope_reject_of_invalid_range
    (cfg : ContractConfig) (r : SlabReceipt)
    (hRange : ¬ RangeValid r) :
    verifySlabEnvelope cfg r = false := by
  unfold verifySlabEnvelope SummaryConsistent
  simp [hRange]

theorem verify_slab_envelope_reject_of_bad_count
    (cfg : ContractConfig) (r : SlabReceipt)
    (hCount : ¬ RangeCountMatches r) :
    verifySlabEnvelope cfg r = false := by
  unfold verifySlabEnvelope SummaryConsistent
  simp [hCount]

theorem verify_slab_envelope_reject_of_overflow
    (cfg : ContractConfig) (r : SlabReceipt)
    (hOverflow : ¬ SummaryNoOverflow r) :
    verifySlabEnvelope cfg r = false := by
  unfold verifySlabEnvelope SummaryConsistent
  simp [hOverflow]

theorem verify_slab_reject_of_wrong_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSummary : ¬ SummaryConsistent r) :
    verifySlab cfg r = false := by
  unfold verifySlab verifySlabWithMerkle
  simp [hSummary]

theorem verify_slab_reject_of_bad_merkle
    (cfg : ContractConfig) (r : SlabReceipt)
    (hMerkle : ¬ MerklePathValid r) :
    verifySlab cfg r = false := by
  unfold verifySlab verifySlabWithMerkle
  simp [hMerkle]

end Coh.Contract



/-- Correctness: verifySlab returns true iff all structural and policy invariants hold. --/
theorem rv_slab_correctness
    (cfg : ContractConfig) (r : SlabReceipt) :
    verifySlab cfg r = true <->
      (SlabReceipt.ValidSchema cfg r ^ SummaryConsistent r ^ MerklePathValid r) := by
  unfold verifySlab verifySlabWithMerkle
  simp
