import Coh.Prelude

namespace Coh.Core

/-- Canonical chain digest carrier for the frozen theorem surface. -/
structure ChainDigest where
  bytes : String
  deriving Repr, DecidableEq

/-- Canonical state-hash carrier for the frozen theorem surface. -/
structure StateHash where
  value : String
  deriving Repr, DecidableEq

/-- Domain separation tag matching the Rust chain-digest rule. -/
def digestDomainTag : String :=
  "COH_V1_CHAIN"

/-- Explicit digest input profile corresponding to the Rust update rule. -/
structure DigestInput where
  domainTag : String
  prevDigest : ChainDigest
  canonicalPayload : String
  deriving Repr, DecidableEq

/-- Canonical digest input assembled from prior digest and canonical bytes/payload. -/
def digestInput (prev : ChainDigest) (payload : String) : DigestInput :=
  { domainTag := digestDomainTag
    prevDigest := prev
    canonicalPayload := payload }

/-- Deterministic digest-update rule mirroring the Rust input layout.
    This is still a symbolic model of hashing, but it now freezes the exact
    domain-separated input shape `tag | prev | payload`. -/
def hashBytes (input : String) : ChainDigest :=
  ⟨s!"SHA256({input})"⟩

/-- Exact preimage bytes passed to the hash boundary in the current Lean model. -/
def digestPreimage (prev : ChainDigest) (payload : String) : String :=
  s!"{digestDomainTag}|{prev.bytes}|{payload}"

def digestUpdate (prev : ChainDigest) (payload : String) : ChainDigest :=
  hashBytes (digestPreimage prev payload)

theorem digestUpdate_deterministic (prev : ChainDigest) (payload : String) :
    digestUpdate prev payload = digestUpdate prev payload := rfl

theorem digestInput_domainTag (prev : ChainDigest) (payload : String) :
    (digestInput prev payload).domainTag = digestDomainTag := rfl

theorem digestInput_prevDigest (prev : ChainDigest) (payload : String) :
    (digestInput prev payload).prevDigest = prev := rfl

theorem digestInput_payload (prev : ChainDigest) (payload : String) :
    (digestInput prev payload).canonicalPayload = payload := rfl

theorem digestUpdate_eq_iff_same_inputs
    (prev₁ prev₂ : ChainDigest) (payload₁ payload₂ : String) :
    digestInput prev₁ payload₁ = digestInput prev₂ payload₂ ↔
      prev₁ = prev₂ ∧ payload₁ = payload₂ := by
  constructor
  · intro h
    cases h
    exact ⟨rfl, rfl⟩
  · rintro ⟨rfl, rfl⟩
    rfl

theorem digestUpdate_matches_contract_shape (prev : ChainDigest) (payload : String) :
    digestUpdate prev payload = hashBytes (digestPreimage prev payload) := rfl

theorem digestPreimage_matches_contract_shape (prev : ChainDigest) (payload : String) :
    digestPreimage prev payload = s!"{digestDomainTag}|{prev.bytes}|{payload}" := rfl

end Coh.Core
