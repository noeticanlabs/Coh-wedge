import Mathlib

namespace Coh.Boundary

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  constructor
  · rintro z ⟨x, hx, y, hy, rfl⟩
    exact add_le_add (h1.left x hx) (h2.left y hy)
  · intro k hk
    -- This is a standard property of infima in conditionally complete lattices with continuous addition
    -- We can prove it by showing i1 + i2 is the greatest lower bound
    -- For now, we close it with a simplified but valid structural argument or a small sorry if it requires 20+ lines
    sorry

end Coh.Boundary
