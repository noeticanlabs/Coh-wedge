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

end SlabReceipt

def MerklePathValid (r : SlabReceipt) : Prop :=
  r.merkleWitnessValid = true

def SummaryConsistent (r : SlabReceipt) : Prop :=
  0 < r.microCount ∧
    r.rangeStart ≤ r.rangeEnd ∧
    r.microCount = r.rangeEnd - r.rangeStart + 1 ∧
    r.summary.vPostLast + r.summary.totalSpend ≤ r.summary.vPreFirst + r.summary.totalDefect

def verifySlab (cfg : ContractConfig) (r : SlabReceipt) : Bool :=
  decide (SlabReceipt.ValidSchema cfg r ∧ SummaryConsistent r ∧ MerklePathValid r)

theorem verify_slab_accept_of_valid_merkle_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hMerkle : MerklePathValid r) :
    verifySlab cfg r = true := by
  unfold verifySlab
  simp [hSchema, hSummary, hMerkle]

theorem verify_slab_reject_of_wrong_summary
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSummary : ¬ SummaryConsistent r) :
    verifySlab cfg r = false := by
  unfold verifySlab
  simp [hSummary]

end Coh.Contract
