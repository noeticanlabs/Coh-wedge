import Mathlib

namespace Coh.Boundary

/-- Extended Non-Negative Rationals: NNRat with an infinity element ⊤ -/
abbrev ENNRat := WithTop NNRat

/--
A set s has rational infimum i if:
1. i is a lower bound of s
2. Any lower bound k satisfies k ≤ i
-/
structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/--
Theorem: The infimum of the pairwise sum of two sets is the sum of their infima.
`inf (s1 + s2) = inf s1 + inf s2`
-/
theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  -- Proof: Show that i1 + i2 is the infimum of the pairwise sum set
  -- Part 1: i1 + i2 is a lower bound
  constructor
  · intro z hz
    rcases hz with ⟨x, hx₁, y, hx₂, rfl⟩
    have h₁ := h1.left x hx₁
    have h₂ := h2.left y hx₂
    exact add_le_add h₁ h₂
  -- Part 2: i1 + i2 is the greatest lower bound
  intro k hk
  -- k is a lower bound of s1 + s2
  -- Need to show k ≤ i1 + i2
  have hk₁ : ∀ x ∈ s1, k ≤ x + i2 := by
    intro x hx
    have h := hk (x + i2)
    refine h ?_
    exact ⟨x, hx, i2, h2.left i2 (by trivial), rfl⟩
  have hk₂ : ∀ y ∈ s2, k - i2 ≤ y := by
    intro y hy
    have key_ineq := hk (i1 + y)
    apply key_ineq
    exact ⟨i1, h1.left i1 (by trivial), y, hy, rfl⟩
  have h₁ := h1.greatest (fun x hx => (hk₁ x hx).1)
  have h₂ := h2.greatest (fun y hy => (hk₂ y hy).1)
  exact add_le_add h₁ h₂

end Coh.Boundary
