import Coh.Kernel.T1_Category
import Coh.Slack.T2_OplaxBridge

namespace Coh.Trace

universe u

/-- [LEMMA-NEEDED] Functor Adm -/
opaque Functor_Adm {X : Type u} (C : Coh.Kernel.StrictCoh X) : Coh.Kernel.SmallCategory C.obj

/-- [LEMMA-NEEDED] Embedding K -/
opaque embeddingK {X : Type u} (C : Coh.Kernel.SmallCategory X) : Coh.Kernel.SmallCategory X

end Coh.Trace
