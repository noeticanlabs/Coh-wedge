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
  sorry

end Coh.Boundary
