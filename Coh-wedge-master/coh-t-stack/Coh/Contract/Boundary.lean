import Mathlib.Data.ByteArray
import Coh.Category.GovCat
import Coh.Contract.Micro
import Coh.Contract.RejectCode
import Coh.Contract.Canon

/-!
# Coh.Contract.Boundary

Canon-bound contract layer interfaces and structural verifiers using ByteArray
digests. This module freezes the boundary-facing verifier substrate without
modifying the categorical core. It introduces:

- `Digest := ByteArray`
- `CanonProfile` with `domainTag`, `jcsEncode`, `hashBytes`
- `chainUpdate` combining domain tag, prior digest, and canonical receipt bytes
- `BoundaryVerifier` interface (`RV : Digest → Receipt → Digest → Digest → Decision Code`)
- Structural predicates for micro receipts: `microSchemaOk`, `microStateLinkOk`,
  `microChainOk`
- `verifyMicroStruct` returning `Decision RejectCode`

This is the Option B freeze: contract becomes canon-grade while keeping the
category and core stable. Adapters to the categorical layer can be added
without churn.
-/

namespace Coh.Contract

open Coh

/-- Canon-bound digest at the verifier boundary. -/
abbrev Digest := ByteArray

/-- Minimal canon profile interface for contract-bound Coh objects. -/
structure CanonProfile where
  profileId : String
  domainTag : ByteArray
  jcsEncode : String → ByteArray
  hashBytes : ByteArray → Digest
deriving Repr

/-- Canonical chain update rule: domain-separated hashing of the tuple
    `(domainTag, prev, receiptBytes)` encoded as a flat byte sequence. -/
def chainUpdate (cp : CanonProfile) (prev : Digest) (receiptBytes : ByteArray) : Digest :=
  cp.hashBytes (cp.domainTag ++ prev ++ receiptBytes)

/-- Contract-level boundary verifier shape. -/
structure BoundaryVerifier (Receipt Code : Type) where
  RV : Digest → Receipt → Digest → Digest → Decision Code

/-- Bridge conversions from legacy core carriers into boundary digests. -/
def fromCoreDigest (d : Coh.Core.ChainDigest) : Digest := d.bytes.toUTF8
def fromCoreState  (s : Coh.Core.StateHash) : Digest := s.value.toUTF8

/-- Structural schema/policy/canon checks for Micro. -/
structure BoundaryConfig where
  base          : ContractConfig
  expectedPolicyHash : String
  canonProfile  : CanonProfile
deriving Repr

def microSchemaOk (bc : BoundaryConfig) (r : MicroReceipt) : Prop :=
  r.schemaId = bc.base.microSchema ∧
  r.version = bc.base.microVersion ∧
  r.objectId ≠ "" ∧
  r.canonProfileHash = bc.base.canonProfileHash ∧
  r.policyHash = bc.expectedPolicyHash

def microStateLinkOk (prevState nextState : Digest) (r : MicroReceipt) : Prop :=
  fromCoreState r.stateHashPrev = prevState ∧
  fromCoreState r.stateHashNext = nextState

def microChainOk
    (bc : BoundaryConfig)
    (encode : MicroReceipt → ByteArray)
    (prevChainDigest : Digest)
    (r : MicroReceipt) : Prop :=
  fromCoreDigest r.chainDigestPrev = prevChainDigest ∧
  fromCoreDigest r.chainDigestNext = chainUpdate bc.canonProfile prevChainDigest (encode r)

/-- Universal structural verifier kernel for micro receipts. -/
def verifyMicroStruct
    (bc : BoundaryConfig)
    (encode : MicroReceipt → ByteArray)
    (prevState nextState prevChainDigest : Digest)
    (r : MicroReceipt) : Decision RejectCode := by
  classical
  haveI := Classical.decEq ByteArray
  -- Schema/canon/policy
  if hSchema : microSchemaOk bc r then
    -- State linkage
    if hState : microStateLinkOk prevState nextState r then
      -- Chain update
      if hChain : microChainOk bc encode prevChainDigest r then
        exact Decision.accept
      else
        exact Decision.reject RejectCode.rejectChainDigest
    else
      exact Decision.reject RejectCode.rejectStateHashLink
  else
    exact Decision.reject RejectCode.rejectSchema

/-- Canonical encoder from a Micro receipt to boundary bytes using JCS. -/
def encodeMicroJCS (r : MicroReceipt) : ByteArray :=
  (canonicalMicroJson r).toUTF8

end Coh.Contract
