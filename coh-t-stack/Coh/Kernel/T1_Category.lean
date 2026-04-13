import Coh.Kernel.Receipt
import Coh.Kernel.Verifier
import Mathlib.Tactic.Linarith

namespace Coh.Kernel

universe u

/-!
# T1: Categorical Extraction & Physical Persistence

## Categorical Ledger (Strict Coh ⇒ Category)
Claim: The admissible fragment of a strict Coh system forms a small category.
Proof: A strict Coh system is defined with objects, candidate transitions, partial
composition, identities, and a verifier (RV). The admissible fragment only includes
transitions where RV(f)=1. The proof verifies all category axioms: Hom-sets are
valid subsets; composition is well-defined because admissibility is closed under
composition; identities exist because RV(1_x)=1; and associativity and unit laws
hold because they are perfectly inherited from the ambient transition system.

## Physics Spine (Persistence)
Claim: Nontrivial lawful propagating modes exist.
This establishes the foundation for the governed substrate carrying enduring
signal-bearing structures (a nonzero admissible propagation sector).
-/

/-- Extracting the category of objects and lawful transitions. -/
structure LawfulTransition (X : Type u) where
  source : X
  target : X
  receipt : Receipt
  is_lawful : Lawful receipt

/-- T1 Category Definition. -/
def CatObj (X : Type u) : Type u := X

/-- Identity transition is lawful. -/
def transition_id (_ : CatObj X) : Receipt :=
  { pre := 0, post := 0, spend := 0, defect := 0, authority := 0 }

lemma transition_id_lawful (x : CatObj X) : Lawful (transition_id x) := by
  unfold Lawful transition_id
  simp

/-- Composition of lawful transitions is lawful. -/
def transition_comp (r2 r1 : Receipt) : Receipt :=
  { pre := r1.pre, post := r2.post, spend := r1.spend + r2.spend,
    defect := r1.defect + r2.defect, authority := r1.authority + r2.authority }

theorem transition_comp_lawful (r2 r1 : Receipt) (h2 : Lawful r2) (h1 : Lawful r1)
    (h_compat : r1.post = r2.pre) : Lawful (transition_comp r2 r1) := by
  unfold Lawful transition_comp at *
  dsimp at *
  linarith

/-- T1: Strict Coh System definition
    A strict Coh system consists of:
    - A type of objects X
    - A family of candidate transitions Hom : X → X → Type
    - Each transition carries a Receipt
    - A verifier RV : Hom x y → Bool that marks transitions as admissible
    - The verifier must be sound: RV f = true → Lawful (receipt of f)
-/
structure StrictCoh (X : Type u) where
  /-- Object type -/
  obj : Type u
  /-- Morphism type: source, target, and receipt -/
  Hom : obj → obj → Type u
  /-- Each morphism has an associated receipt -/
  receipt : {x y : obj} → Hom x y → Receipt
  /-- Identity morphism -/
  id : (x : obj) → Hom x x
  /-- Composition -/
  comp : {x y z : obj} → Hom y z → Hom x y → Hom x z
  /-- Verifier: checks if transition is admissible (lawful) -/
  RV : {x y : obj} → Hom x y → Bool
  /-- RV accepts only lawful transitions (soundness) -/
  rv_sound : ∀ {x y} (f : Hom x y), RV f = true → Lawful (receipt f)
  /-- Identity is admissible -/
  rv_id : ∀ (x : obj), RV (id x) = true
  /-- Composition of admissible transitions is admissible -/
  rv_comp : ∀ {x y z} (g : Hom y z) (f : Hom x y),
            RV g = true → RV f = true → RV (comp g f) = true
  /-- Category axioms for the ambient (non-admissible) system -/
  id_comp : ∀ {x y} (f : Hom x y), comp (id y) f = f
  comp_id : ∀ {x y} (f : Hom x y), comp f (id x) = f
  assoc : ∀ {w x y z} (f : Hom w x) (g : Hom x y) (h : Hom y z),
          comp h (comp g f) = comp (comp h g) f

/-- Small Category structure extracted from Strict Coh's admissible fragment -/
structure SmallCategory (X : Type u) where
  Hom : X → X → Type u
  id : (x : X) → Hom x x
  comp : {x y z : X} → Hom y z → Hom x y → Hom x z
  id_comp : ∀ {x y} (f : Hom x y), comp (id y) f = f
  comp_id : ∀ {x y} (f : Hom x y), comp f (id x) = f
  assoc : ∀ {w x y z} (f : Hom w x) (g : Hom x y) (h : Hom y z),
          comp h (comp g f) = comp (comp h g) f

/-- T1: the admissible fragment of a strict Coh system carries a small-category structure.
    This is the Categorical Ledger proof of T1. -/
def T1_StrictCoh_to_Category {X : Type u} (C : StrictCoh X) :
    SmallCategory C.obj := {
  Hom := fun x y => { f : C.Hom x y // C.RV f = true }
  id := fun x => ⟨C.id x, C.rv_id x⟩
  comp := fun g f => ⟨C.comp g.val f.val, C.rv_comp g.val f.val g.property f.property⟩
  id_comp := fun f => Subtype.ext (C.id_comp f.val)
  comp_id := fun f => Subtype.ext (C.comp_id f.val)
  assoc := fun f g h => Subtype.ext (C.assoc f.val g.val h.val)
}

end Coh.Kernel
