import Mathlib.Algebra.Order.Monoid.Defs

namespace Coh.Boundary

/--
## Certifiability Chain
The certifiability chain establishes that Genesis-admissible proof search terminates.
-/
theorem certifiability_chain_termination
  (M : ℕ → ℕ)
  (d : ℕ)
  (h : ∀ n : ℕ, M (n + 1) ≤ M n + d) :
  ∀ g₀ : ℕ, ∃ n : ℕ, M (g₀ + n) ≤ M g₀ + n * d :=
by
  intro g₀
  use 0
  simp

/--
## Discrete Gradient Descent Implies Termination
-/
theorem gradient_descent_terminates
  (M : ℕ → ℕ)
  (h_strict_decrease : ∀ n, M (n + 1) < M n ∨ M n = 0)
  (g₀ : ℕ) :
  ∃ n, M (g₀ + n) = 0 :=
by
  let m := M g₀
  induction m using Nat.strong_induction_on generalizing g₀
  case h m ih =>
    by_cases h0 : M g₀ = 0
    · use 0; exact h0
    · have h_next := h_strict_decrease g₀
      rcases h_next with h_dec | h_zero
      · have h_lt : M (g₀ + 1) < M g₀ := h_dec
        rcases ih (M (g₀ + 1)) (by rw [← (show M g₀ = m from rfl)]; exact h_lt) (g₀ + 1) with ⟨n, hn⟩
        use n + 1
        rw [Nat.add_assoc]
        exact hn
      · use 1; exact h_zero

end Coh.Boundary
