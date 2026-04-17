import Coh.Kernel.T1_Category
import Coh.Slack.T2_OplaxBridge

namespace Coh.Trace

universe u

-- The admissible fragment category extracted from a StrictCoh.
-- This extracts exactly the morphisms where RV = true (admissible transitions).
-- Functoriality follows from StrictCoh's rv_comp and rv_id axioms.
def Functor_Adm {X : Type u} (C : Coh.Kernel.StrictCoh X) :
    Coh.Kernel.SmallCategory C.obj :=
  Coh.Kernel.T1_StrictCoh_to_Category C

-- The embedding K lifts a SmallCategory (the T1 ledger) into a full SmallCategory
-- by using the T2 bridge to convert to StrictCoh then back to SmallCategory.
-- This establishes the "Macro Slab" categorical embedding from T3.
def embeddingK {X : Type u} (C : Coh.Kernel.SmallCategory X) :
    Coh.Kernel.SmallCategory X :=
  Coh.Slack.T2_Category_to_StrictCoh C

end Coh.Trace
