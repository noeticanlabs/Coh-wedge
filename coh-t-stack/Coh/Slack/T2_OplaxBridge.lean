import Coh.Kernel.T1_Category

namespace Coh.Slack

universe u

/-- [LEMMA-NEEDED] T2 Bridge -/
opaque T2_Category_to_StrictCoh {X : Type u} (C : Coh.Kernel.SmallCategory X) : Coh.Kernel.SmallCategory X

/-- [LEMMA-NEEDED] T5 Functor -/
opaque T5_Embedding_is_Functor {X : Type u} (C : Coh.Kernel.SmallCategory X) : Coh.Kernel.StrictCoh X

end Coh.Slack
