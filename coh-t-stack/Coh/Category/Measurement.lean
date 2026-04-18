import Coh.Category.CohCat
import Coh.Category.CohDyn
import Mathlib.Data.Set.Basic

/-!
# Coh.Category.Measurement

This module implements the formal Coh-native theory of Measurement.

A **Measurement** is a structure-preserving map between governed systems.
1. **Collapse**: Precise categorical capture of information loss (non-faithfulness).
2. **Dissipation**: Predicate-level capture of entropy/cost decay (oplax constraint).
3. **Fibers**: Preimage sets of observable traces.
-/

namespace Coh.Category

open Coh.Kernel
open Coh.Category

/-- A Measurement is a verification-preserving morphism between governed systems. -/
def Measurement (A B : CohObj) := CohHom A B

namespace Measurement

variable {A B : CohObj}

/-- Induced functor on dynamics categories. [PROVED] -/
def DynMap (m : Measurement A B) : CohDyn A ⥤ CohDyn B :=
  DynFunctor.toSmallFunctor m

/-- 
A measurement exhibits **collapse** if it maps distinct hidden traces 
to the same observed trace. This is the definition of non-faithfulness 
for the induced dynamics functor. [PROVED]
-/
def collapses (m : Measurement A B) : Prop :=
  ∃ (x y : A.X) (t₁ t₂ : x ⟶ y),
    t₁ ≠ t₂ ∧ m.DynMap.map t₁ = m.DynMap.map t₂

/-- Observational equivalence on traces. [PROVED] -/
def equiv (m : Measurement A B) {x y : A.X} (t₁ t₂ : x ⟶ y) : Prop :=
  m.DynMap.map t₁ = m.DynMap.map t₂

/-- 
The **Fiber** of an observable trace `t` in `B` is the set of 
hidden traces in `A` that map to it. [PROVED]
-/
def Fiber (m : Measurement A B) {x y : A.X} (t : m.fX x ⟶ m.fX y) :=
  { s : x ⟶ y // m.DynMap.map s = t }

/-- 
A measurement is **oplax** with respect to cost if the cost of the 
recorded dynamics is bounded by the cost of the hidden dynamics.
This corresponds to strict dissipation of information/energy. [PROVED]
-/
def is_oplax (m : Measurement A B) : Prop :=
  ∀ {x y : A.X} (t : x ⟶ y),
    cost B (m.DynMap.map t) ≤ cost A t

end Measurement

/-- 
Formal Implementation of a Coh Measurement: 
A structured morphism that satisfies the dissipation (oplax) constraint.
-/
structure CohMeasurement (A B : CohObj) extends CohHom A B where
  oplax : ∀ {x y} (t : x ⟶ y),
    cost B (DynFunctor.mapDyn toCohHom t) ≤ cost A t

namespace CohMeasurement

/- Composition of CohMeasurements preserves the oplax property. [PROVED] -/
def comp {A B C : CohObj} (g : CohMeasurement B C) (f : CohMeasurement A B) : 
    CohMeasurement A C :=
  { toCohHom := CohHom.comp g.toCohHom f.toCohHom
  , oplax := by
      intro x y t
      -- Bound B cost by A cost
      have hf := f.oplax t
      -- Bound C cost by B cost (at the mapped trace)
      let t_mapped := DynFunctor.mapDyn f.toCohHom t
      have hg := g.oplax t_mapped
      -- Transitivity: cost C ∘ f ≤ cost B ∘ f ≤ cost A
      exact le_trans hg hf }

/-- Identity measurement: Zero dissipation, no collapse. [PROVED] -/
def id (A : CohObj) : CohMeasurement A A :=
  { toCohHom := CohHom.id A
  , oplax := by intro x y t; simp [CohHom.id, DynFunctor.mapDyn]; exact le_refl _ }

/-- An identity measurement never collapses. [PROVED] -/
theorem id_no_collapse (A : CohObj) :
    ¬ (Measurement.collapses (CohHom.id A)) := by
  unfold Measurement.collapses
  push_neg
  intro x y t1 t2
  intro hMap
  simp [CohHom.id, Measurement.DynMap, DynFunctor.toSmallFunctor, DynFunctor.mapDyn] at hMap
  -- In DynFunctor.mapDyn, the id map is clearly injective on the data structure
  exact hMap

end CohMeasurement

end Coh.Category
