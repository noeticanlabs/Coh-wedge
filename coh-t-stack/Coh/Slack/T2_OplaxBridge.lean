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
    rv_sound := fun f _ => by simp only [R, Coh.Kernel.Lawful, zero_add, and_self],
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
