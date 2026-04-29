import Mathlib.Data.NNRat.Defs
import Mathlib.Algebra.Order.Monoid.WithTop
import Mathlib.Order.WithBot
import Mathlib.Order.ConditionallyCompleteLattice.Basic

namespace Coh.Boundary

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  lower : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

lemma isRationalInf_iff_isGLB (s : Set ENNRat) (i : ENNRat) :
  IsRationalInf s i ↔ IsGLB s i := by
  constructor
  · rintro ⟨h1, h2⟩
    exact ⟨h1, h2⟩
  · rintro ⟨h1, h2⟩
    exact ⟨h1, h2⟩

#check IsGLB
#check IsGLB.add

end Coh.Boundary
