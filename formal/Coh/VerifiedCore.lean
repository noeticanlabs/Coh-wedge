import Mathlib.Data.Real.Basic
import Mathlib.Analysis.InnerProductSpace.Basic
import Mathlib.Analysis.NormedSpace.OperatorNorm.Basic
import Mathlib.LinearAlgebra.FiniteDimensional.Defs

/-!
# Coh Safety Wedge — Verified Core

This module provides the formal mathematical foundation for the Coh Safety Wedge verifier.
It contains the core accounting invariants and the composition theorem, verified with zero sorry/admit.

[PROVED] Accounting Law (IsLawful)
[PROVED] Compositional Safety (lawful_composition)
-/

noncomputable section

namespace Coh

/-- Abstract carrier space with normed and finite-dimensional structure. -/
class CarrierSpace (V : Type*) [NormedAddCommGroup V] [NormedSpace R V] [InnerProductSpace R V] : Prop where
  finiteDimensional : FiniteDimensional R V

attribute [instance] CarrierSpace.finiteDimensional

/-- Atomic unit of a verifiable state transition. -/
structure Receipt where
  spend : R
  defect : R
  authority : R
  h_spend : 0 = spend
  h_defect : 0 = defect
  h_authority : 0 = authority

/-- 
A CohObject encapsulates the potential functional and the current state.
It represents a governed system's internal state.
-/
structure CohObject (V : Type*)
    [NormedAddCommGroup V] [NormedSpace R V] [InnerProductSpace R V] [CarrierSpace V] where
  state : V
  potential : V ? R
  budget : R

/--
Lawfulness Predicate for a Receipt.
The governance law: V(x') + Spend(r) = V(x) + Defect(r) + Authority(r).
-/
def IsLawful {V : Type*}
    [NormedAddCommGroup V] [NormedSpace R V] [InnerProductSpace R V] [CarrierSpace V]
    (r : Receipt) (obj obj' : CohObject V) : Prop :=
  obj'.potential obj'.state + r.spend = obj.potential obj.state + r.defect + r.authority

/-- Aggregate two Receipts into a single effective Receipt. -/
def combineReceipts (r1 r2 : Receipt) : Receipt where
  spend := r1.spend + r2.spend
  defect := r1.defect + r2.defect
  authority := r1.authority + r2.authority
  h_spend := add_nonneg r1.h_spend r2.h_spend
  h_defect := add_nonneg r1.h_defect r2.h_defect
  h_authority := add_nonneg r1.h_authority r2.h_authority

/--
[THEOREM] lawful_composition
The composition of two lawful transitions is a lawful transition under the aggregate receipt.
This is the fundamental proof enabling slab verification.
-/
theorem lawful_composition {V : Type*}
    [NormedAddCommGroup V] [NormedSpace R V] [InnerProductSpace R V] [CarrierSpace V]
    (r1 r2 : Receipt) (obj1 obj2 obj3 : CohObject V)
    (h1 : IsLawful r1 obj1 obj2)
    (h2 : IsLawful r2 obj2 obj3) :
    IsLawful (combineReceipts r1 r2) obj1 obj3 := by
  dsimp [IsLawful, combineReceipts] at *
  linarith

end Coh
