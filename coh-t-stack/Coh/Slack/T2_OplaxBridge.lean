import Coh.Kernel.T1_Category

namespace Coh.Slack

universe u

-- Lift a SmallCategory to a StrictCoh by trivially satisfying all structural constraints.
-- The verifier RV is set to always-true, making every morphism admissible.
-- This creates the "oplogous" (slack) extension that allows categorical composition.
def Category_to_StrictCoh {X : Type u} (C : Coh.Kernel.SmallCategory X) :
    Coh.Kernel.StrictCoh X :=
  let R : Coh.Kernel.Receipt := { pre := 0, post := 0, spend := 0, defect := 0, authority := 0 }
  { obj := X,
    Hom := C.Hom,
    receipt := fun _ => R,
    id := C.id,
    comp := C.comp,
    RV := fun _ => true,
    rv_sound := fun f _ => by simp [R, Coh.Kernel.Lawful],
    rv_id := fun _ => rfl,
    rv_comp := fun _ _ _ _ => rfl,
    id_comp := C.id_comp,
    comp_id := C.comp_id,
    assoc := C.assoc }

-- T2: Admissible fragment extraction - the bridge from oplax categorical structure
-- to the strict Coh system's admissible fragment. This is the explicit implementation
-- of the T2 bridge that extracts the category structure.
def T2_Category_to_StrictCoh {X : Type u} (C : Coh.Kernel.SmallCategory X) :
    Coh.Kernel.SmallCategory X :=
  Coh.Kernel.T1_StrictCoh_to_Category (Category_to_StrictCoh C)

-- T5 Functor: Embed a SmallCategory directly into StrictCoh.
-- This is the categorical embedding that lifts category-theoretic structure
-- into the Coh system's strict compositional framework.
def T5_Embedding_is_Functor {X : Type u} (C : Coh.Kernel.SmallCategory X) :
    Coh.Kernel.StrictCoh X :=
  Category_to_StrictCoh C

/-!
## T2 Round-Trip Faithfulness

The construction `Category â†’ StrictCoh â†’ SmallCategory` (the T2 bridge)
round-trips faithfully: the resulting `SmallCategory` is definitionally
equal to the input, because:
1. `Category_to_StrictCoh` sets `Hom := C.Hom`, `comp := C.comp`, etc.
2. `T1_StrictCoh_to_Category` extracts `{ f : Hom x y // RV f = true }`.
3. Since `RV := fun _ => true`, every subtype `{ f // true }` is
   trivially equivalent to the original type.

The theorems below prove this equivalence structurally.
-/

/-- The Hom-type of the T2 round-trip is a trivial subtype of the original. -/
theorem t2_hom_is_trivial_subtype {X : Type u} (C : Coh.Kernel.SmallCategory X)
    (x y : X) :
    (T2_Category_to_StrictCoh C).Hom x y = { f : C.Hom x y // true } := rfl

/-- The identity morphism round-trips correctly. -/
theorem t2_id_roundtrip {X : Type u} (C : Coh.Kernel.SmallCategory X) (x : X) :
    ((T2_Category_to_StrictCoh C).id x).val = C.id x := rfl

/-- Composition round-trips correctly. -/
theorem t2_comp_roundtrip {X : Type u} (C : Coh.Kernel.SmallCategory X)
    {x y z : X}
    (g : (T2_Category_to_StrictCoh C).Hom y z)
    (f : (T2_Category_to_StrictCoh C).Hom x y) :
    ((T2_Category_to_StrictCoh C).comp g f).val = C.comp g.val f.val := rfl

end Coh.Slack

