import Coh.Kernel.T1_Category

/-!
# Coh.Category: Base category of governed systems (objects with V and RV)

Objects carry:
- a state type X
- a receipt type R
- a potential V : X → Nat (nonnegative discrete potential for ledger alignment)
- a verifier RV : X → R → X → Bool (discrete acceptance predicate)

Morphisms f : A ⟶ B are pairs (fX, fR) that preserve acceptance:
  RV_A x r x' = true → RV_B (fX x) (fR r) (fX x') = true

This file builds a SmallCategory structure (in the style of T1) over these
objects and homomorphisms, so users can reason categorically about translators
between governed systems.
-/

namespace Coh.Category

universe u v

/- Base objects: (X, R, V, RV) -/
structure CohObj where
  X  : Type u
  R  : Type v
  V  : X → Nat
  RV : X → R → X → Bool

/- Verification-preserving morphisms between base objects -/
structure CohHom (A B : CohObj) where
  fX : A.X → B.X
  fR : A.R → B.R
  preserves : ∀ {x x' : A.X} {r : A.R},
    A.RV x r x' = true →
    B.RV (fX x) (fR r) (fX x') = true

namespace CohHom

/- Identity morphism -/
def id (A : CohObj) : CohHom A A :=
  { fX := id, fR := id, preserves := by intro x x' r h; simpa using h }

/- Composition of morphisms -/
def comp {A B C : CohObj} (g : CohHom B C) (f : CohHom A B) : CohHom A C :=
  { fX := fun x => g.fX (f.fX x)
  , fR := fun r => g.fR (f.fR r)
  , preserves := by
      intro x x' r h
      have hB : B.RV (f.fX x) (f.fR r) (f.fX x') = true := f.preserves h
      exact g.preserves hB }

end CohHom

/- SmallCategory instance following Coh.Kernel.SmallCategory style -/
open Coh.Kernel

def CohCat : SmallCategory CohObj :=
  { Hom := fun A B => CohHom A B
  , id := fun A => CohHom.id A
  , comp := fun g f => CohHom.comp g f
  , id_comp := by
      intro A B f; cases f; rfl
  , comp_id := by
      intro A B f; cases f; rfl
  , assoc := by
      intro A B C D f g h; cases f; cases g; cases h; rfl }

end Coh.Category
