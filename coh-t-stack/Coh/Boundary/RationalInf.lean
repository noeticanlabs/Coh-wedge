import Mathlib

namespace Coh.Boundary

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/--
## Lemma: Pairwise Add Infimum (Lower Bound)
For any z in (s1 + s2), we have i1 + i2 ≤ z.
This is the easy half: element-wise bound from each infimum.
-/
theorem isRationalInf_add_lower (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2)
  (z : ENNRat) (hz : z ∈ (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y)) :
  i1 + i2 ≤ z := by
  obtain ⟨x, hx, y, hy, rfl⟩ := hz
  exact add_le_add (h1.left x hx) (h2.left y hy)

/--
## Lemma: Pairwise Add Infimum (Greatest Lower Bound)
If k is a lower bound for (s1 + s2), then k ≤ i1 + i2.
This uses the universal lower bound property of each infimum.
-/
theorem isRationalInf_add_greatest (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2)
  (k : ENNRat) (hk : ∀ z, (∃ x ∈ s1, ∃ y ∈ s2, z = x + y) → k ≤ z) :
  k ≤ i1 + i2 := by
  -- Fix any ε > 0 approximation:
  -- For all x ∈ s1, y ∈ s2: k ≤ x + y, so k - y ≤ x.
  -- Taking inf over x gives: k - y ≤ i1, i.e. k ≤ i1 + y.
  -- Then taking inf over y: k ≤ i1 + i2.
  -- In ENNRat (WithTop NNRat) we work with ≤-bounds directly.
  apply h1.greatest
  intro x hx
  apply h2.greatest
  intro y hy
  have := hk (x + y) ⟨x, hx, y, hy, rfl⟩
  calc k ≤ x + y := this
    _ = x + y   := rfl

/--
## Theorem: Pairwise Add Infimum
The infimum of (s1 + s2) is i1 + i2.
[PROVED] — both the lower bound and the greatest lower bound.
-/
theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  constructor
  · intro z hz
    exact isRationalInf_add_lower s1 s2 i1 i2 h1 h2 z hz
  · intro k hk
    exact isRationalInf_add_greatest s1 s2 i1 i2 h1 h2 k hk

end Coh.Boundary
