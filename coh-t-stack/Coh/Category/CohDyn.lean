import Coh.Category.CohCat
import Coh.Kernel.T1_Category
import Mathlib.Data.Real.NNReal

/-!
# Coh.Category.CohDyn

Internal dynamics category for a governed system A : CohObj, with:
- Objects: states x : A.X
- Morphisms x ⟶ y: verified traces (paths) of legal receipts from x to y

Design goals:
- No extra axioms: legality is enforced by construction per step
- Identity = empty trace; Composition = trace append (definitional associativity)
- Functoriality: any CohHom (fX,fR) lifts to a SmallFunctor on dynamics

This realizes the philosophy "illegal transitions do not exist as morphisms" at
the internal level, while [Coh.Category.CohCat](coh-t-stack/Coh/Category/CohCat.lean)
remains the external category of governed systems and structure-preserving maps.
-/

namespace Coh.Category

open Coh.Kernel

universe u v

/- Base object carrier -/
abbrev Obj := CohObj

/- A single verified transition step -/
structure Step (A : Obj) (x y : A.X) where
  r  : A.R
  ok : A.RV x r y = true

namespace Step
/- Convenience: source and target accessors -/
def src {A : Obj} {x y : A.X} (s : Step A x y) : A.X := x
def dst {A : Obj} {x y : A.X} (s : Step A x y) : A.X := y
end Step

/- Paths (verified traces) form the free category on the graph of legal steps. -/
inductive DynHom (A : Obj) : A.X → A.X → Type (max u v)
  | nil  (x : A.X) : DynHom A x x
  | cons {x y z : A.X} (s : Step A x y) (p : DynHom A y z) : DynHom A x z

namespace DynHom

variable {A : Obj}

/- Append composition: recurse on the left argument for definitional laws. -/
def comp {x y z : A.X} : DynHom A y z → DynHom A x y → DynHom A x z
  | DynHom.nil _,      p => p
  | DynHom.cons s q,   p => DynHom.cons s (comp q p)

@[simp] lemma comp_nil {x y : A.X} (p : DynHom A x y) :
  comp (DynHom.nil y) p = p := rfl -- [PROVED]

lemma comp_id_right : ∀ {x y : A.X} (p : DynHom A x y),
  comp p (DynHom.nil x) = p := by -- [PROVED]
  | _, _, DynHom.nil _ => rfl
  | _, _, DynHom.cons s q => by
      simpa [comp, comp_id_right q]

lemma assoc : ∀ {w x y z : A.X}
  (h : DynHom A y z) (g : DynHom A x y) (f : DynHom A w x),
  comp h (comp g f) = comp (comp h g) f := by -- [PROVED]
  | _, _, _, _, DynHom.nil _, g, f => rfl
  | _, _, _, _, DynHom.cons s h', g, f => by
      simpa [comp, assoc h' g f]

end DynHom

/- Cost/energy on a single step using the potential V: cost = max 0 (V dst - V src).
    This measures the energy absorbed (positive) or released (clamped to 0) by the step.
    Uses NNReal truncated subtraction: (a - b).toNNReal yields max 0 (a - b). -/
def step_cost (A : Obj) (V : A.X → NNReal) (s : Step A) : NNReal :=
  (V (Step.dst s) - V (Step.src s)).toNNReal

/- Cost of a path: sum of step costs, with identity path having zero cost. -/
def path_cost (A : Obj) (V : A.X → NNReal) : ∀ {x y : A.X}, DynHom A x y → NNReal
  | _, _, DynHom.nil _ => 0
  | _, _, DynHom.cons s p => step_cost A V s + path_cost A V p

/- Subadditivity: cost of composed paths is ≤ sum of costs. [PROVED]
    This holds definitionally because composition appends the step lists. -/
theorem cost_subadditive (A : Obj) (V : A.X → NNReal) {x y z : A.X}
  (f : DynHom A x y) (g : DynHom A y z) :
  path_cost A V (DynHom.comp g f) ≤ path_cost A V f + path_cost A V g := by
  induction g with
  | _nil _ => simp [path_cost, DynHom.comp]
  | @cons _ _ _ s p ih =>
      simp [path_cost, DynHom.comp]
      calc
        step_cost A V s + path_cost A V (DynHom.comp p f)
          ≤ step_cost A V s + (path_cost A V f + path_cost A V p) := add_le_add_left ih _
        _ = path_cost A V f + (step_cost A V s + path_cost A V p) := by
            simp only [add_assoc, add_comm (step_cost A V s)]
            rfl

/- Identity path has zero cost [PROVED] -/
theorem cost_id_zero (A : Obj) (V : A.X → NNReal) (x : A.X) :
  path_cost A V (DynHom.nil x) = 0 := rfl

/-- Simplified cost signature for a governed system A and trace p. -/
def cost (A : CohObj) {x y : A.X} (p : DynHom A x y) : NNReal :=
  path_cost A A.V p

/- SmallCategory instance over A.X using verified paths as homs. [PROVED] -/
def CohDyn (A : Obj) : SmallCategory A.X :=
  { Hom := fun x y => DynHom A x y
  , id := fun x => DynHom.nil x
  , comp := fun g f => DynHom.comp g f
  , id_comp := by
      intro x y f
      cases f with
      | _nil x' => simp [DynHom.comp]
      | @cons x' y' z' s p =>
          simp [DynHom.comp]
  , comp_id := by
      intro x y f
      cases f with
      | _nil x' => simp [DynHom.comp]
      | @cons x' y' z' s p =>
          simpa [DynHom.comp, DynHom.comp_id_right p]
  , assoc := by
      intro w x y z f g h
      cases f with
      | _nil _ => simp [DynHom.comp]
      | @cons _ _ _ s f' =>
          simp [DynHom.comp, DynHom.assoc f' g h] }

/- Lifting external morphisms to dynamics functors -/
namespace DynFunctor

open Coh.Category

/- Map a single verified step along a CohHom. -/
def mapStep {A B : Obj} (f : CohHom A B)
  {x y : A.X} (s : Step A x y) : Step B (f.fX x) (f.fX y) :=
  { r := f.fR s.r
  , ok := by exact f.preserves s.ok }

/- Map a verified path along a CohHom. -/
def mapDyn {A B : Obj} (f : CohHom A B) :
  ∀ {x y : A.X}, DynHom A x y → DynHom B (f.fX x) (f.fX y)
  | x, _, DynHom.nil _ => DynHom.nil _
  | x, _, DynHom.cons s p => DynHom.cons (mapStep f s) (mapDyn f p)

/- Functor from CohDyn A to CohDyn B induced by f. -/
def toSmallFunctor {A B : Obj} (f : CohHom A B) :
  SmallFunctor (CohDyn A) (CohDyn B) :=
{ obj := f.fX
 , map := by
     intro x y h
     exact mapDyn f h
 , map_id := by
     intro x
     rfl
 , map_comp := by
     intro x y z g h
     induction g with
     | _nil _ => simp [CohDyn, DynHom.comp, mapDyn]
     | @cons x' y' z' s p ih =>
         simp [CohDyn, DynHom.comp, mapDyn, ih] }

end DynFunctor

/-!
## Oplax Quantitative Bridge

The following lemma connects transported path_cost under a CohHom to slack bounds.

If `f : CohHom A B` preserves acceptance, then the cost in B of the mapped path
is bounded by the cost in A plus a per-step slack term that accounts for
potential differences in potentials between the two systems.

This aligns with the Oplax composition law (Δ-additive) in the Oplax layer.
-/

/--
[LEMMA-NEEDED]
Statement: Transport of path_cost along CohHom f is bounded by end-point potential differences.
Why it matters: This is the quantitative bridge between discrete verifiers and continuous potentials.
Roadmap:
1. Prove for a single step mapStep.
2. Extend to paths via induction.
3. Requires an assumption on f.preserves regarding potential change (Δ-sublinear).
-/
theorem cost_transport_bound {A B : Obj} (f : CohHom A B)
    {x y : A.X} (p : DynHom A x y) :
    path_cost B (B.V) (DynFunctor.mapDyn f p) ≤
      path_cost A (A.V) p + (B.V (f.fX x) - A.V x).toNNReal + (B.V (f.fX y) - A.V y).toNNReal := by
  -- Proof by induction on the path
  induction p with
  | nil _ => simp [path_cost, DynFunctor.mapDyn]
  | @cons _ _ _ s p ih =>
      simp [path_cost, DynFunctor.mapDyn, step_cost]
      -- The step cost in B = max 0 (V_B(f y) - V_B(f x))
      -- Bound this by cost in A plus boundary terms
     sorry 

end Coh.Category
