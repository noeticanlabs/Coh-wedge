import Coh.Prelude

namespace Coh.Core

/-- Canonical chain digest carrier for the frozen theorem surface. -/
inductive ChainDigest where
  | genesis (seed : String)
  | step (prev : ChainDigest) (payload : String)
  deriving Repr, DecidableEq

/-- Canonical state-hash carrier for the frozen theorem surface. -/
structure StateHash where
  value : String
  deriving Repr, DecidableEq

/-- Deterministic digest-update rule used by the contract layer. -/
def digestUpdate (prev : ChainDigest) (payload : String) : ChainDigest :=
  ChainDigest.step prev payload

theorem digestUpdate_deterministic (prev : ChainDigest) (payload : String) :
    digestUpdate prev payload = digestUpdate prev payload := rfl

theorem digestUpdate_eq_iff_same_inputs
    (prev₁ prev₂ : ChainDigest) (payload₁ payload₂ : String) :
    digestUpdate prev₁ payload₁ = digestUpdate prev₂ payload₂ ↔
      prev₁ = prev₂ ∧ payload₁ = payload₂ := by
  constructor
  · intro h
    cases h
    exact ⟨rfl, rfl⟩
  · rintro ⟨rfl, rfl⟩
    rfl

end Coh.Core
