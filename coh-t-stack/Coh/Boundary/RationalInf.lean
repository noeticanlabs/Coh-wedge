import Mathlib.Data.NNRat.Defs
import Mathlib.Algebra.Order.Monoid.WithTop
import Mathlib.Order.WithBot
import Mathlib.Order.ConditionallyCompleteLattice.Basic
import Mathlib.Algebra.Order.Ring.WithTop

namespace Coh.Boundary

/-- Extended Non-Negative Rationals -/
abbrev ENNRat := WithTop NNRat

/-- Extended non-negative rational from NNRat -/
scoped notation "↑" x => (x : ENNRat)

/--
i is the Rational Infimum of set s if it is the Greatest Lower Bound.
-/
structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  lower : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/--
Main theorem target: infimum of pairwise sum of sets equals sum of infima.
This theorem was temporarily simplified pending reconstruction of the epsilon-delta proof.
-/
theorem isRationalInf_pairwise_add {s1 s2 : Set ENNRat} {i1 i2 : ENNRat}
  (h1 : IsRationalInf s1 i1)
  (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  -- This remains to be reconstructed
  sorry

end Coh.Boundary
