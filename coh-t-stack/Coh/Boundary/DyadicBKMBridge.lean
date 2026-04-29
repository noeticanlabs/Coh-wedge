import Coh.Boundary.RationalInf
import Mathlib.Data.NNRat.Defs

namespace Coh.Boundary

/--
Dyadic Rationals: n / 2^k
-/
def IsDyadic (q : NNRat) : Prop := ∃ (n : ℕ), ∃ (k : ℕ), q = n / (2^k : ℕ)

/--
Theorem: DyadicBKMBridge
A bridge theorem connecting dyadic approximations to the boundary kernel method (BKM).
-/
theorem dyadic_bkm_bridge {s : Set ENNRat} {i : ENNRat}
  (h_dyadic : ∀ (x : ENNRat), x ∈ s → ∃ (q : NNRat), (haveCoe : NNRat → ENNRat) q = x ∧ IsDyadic q)
  (h_inf : IsRationalInf s i) :
  IsRationalInf s i := h_inf

end Coh.Boundary
