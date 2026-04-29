import Mathlib.Data.NNRat.Defs
import Mathlib.Order.WithBot
import Mathlib.Order.ConditionallyCompleteLattice.Basic
import Mathlib.Algebra.Order.Ring.WithTop
import Mathlib.Order.CompleteLattice
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring
import Mathlib.Tactic.NormNum
import Mathlib.Data.Rat.Defs

namespace Coh.Boundary

/-- Extended Non-Negative Rationals -/
abbrev ENNRat := WithTop NNRat

/--
i is the Rational Infimum of set s if it is the Greatest Lower Bound.
-/
structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  lower : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/-- Helper: if j is not a lower bound, there exists an element less than it. -/
theorem IsRationalInf.exists_lt_of_lt {s : Set ENNRat} {i : ENNRat} (h : IsRationalInf s i)
  {j : ENNRat} (hj : i < j) : ∃ x ∈ s, x < j := by
  by_contra h'
  push_neg at h'
  have h_lb : ∀ x ∈ s, j ≤ x := fun x hx => h' x hx
  have hj_le_i := h.greatest j h_lb
  exact not_le_of_lt hj hj_le_i

/--
The infimum of a pairwise sum of sets is the sum of their infima.
inf(s1 + s2) = i1 + i2
-/
theorem isRationalInf_pairwise_add {s1 s2 : Set ENNRat} {i1 i2 : ENNRat}
  (h1 : IsRationalInf s1 i1)
  (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  constructor
  · rintro z ⟨x, hx, y, hy, rfl⟩
    exact add_le_add (h1.lower x hx) (h2.lower y hy)
  · intros k hk
    by_contra h_lt
    have h_lt' : i1 + i2 < k := not_le.mp h_lt
    
    if hi1 : i1 = ⊤ then
      rw [hi1, top_add] at h_lt'
      exact not_top_lt h_lt'
    else if hi2 : i2 = ⊤ then
      rw [hi2, add_top] at h_lt'
      exact not_top_lt h_lt'
    else
      obtain ⟨r1, hr1⟩ := WithTop.ne_top_iff_exists.mp hi1
      obtain ⟨r2, hr2⟩ := WithTop.ne_top_iff_exists.mp hi2
      
      if hk_top : k = ⊤ then
        have h_s1_non_top : ∃ x ∈ s1, x < ⊤ := by
          by_contra h_all_top; push_neg at h_all_top
          have : ∀ x ∈ s1, ⊤ ≤ x := fun x hx => top_unique (h_all_top x hx).symm.le
          exact hi1 (top_unique (h1.greatest ⊤ this))
        have h_s2_non_top : ∃ y ∈ s2, y < ⊤ := by
          by_contra h_all_top; push_neg at h_all_top
          have : ∀ y ∈ s2, ⊤ ≤ y := fun y hy => top_unique (h_all_top y hy).symm.le
          exact hi2 (top_unique (h2.greatest ⊤ this))
        
        obtain ⟨x, hx, hx_lt⟩ := h_s1_non_top
        obtain ⟨y, hy, hy_lt⟩ := h_s2_non_top
        have hxy_lt : x + y < ⊤ := WithTop.add_lt_top.mpr ⟨hx_lt, hy_lt⟩
        rw [hk_top] at hk
        exact not_top_lt (hxy_lt.trans_le' (hk (x + y) ⟨x, hx, y, hy, rfl⟩))
      else
        obtain ⟨rk, hrk⟩ := WithTop.ne_top_iff_exists.mp hk_top
        have h_r12_rk : r1 + r2 < rk := by
          rw [← hr1, ← hr2, ← WithTop.coe_add, ← hrk] at h_lt'
          exact WithTop.coe_lt_coe.mp h_lt'
        
        let δ := rk - (r1 + r2)
        have hδ : 0 < δ := tsub_pos_iff_lt.mpr h_r12_rk
        let ε : NNRat := δ / (3 : NNRat)
        have hε_pos : 0 < ε := by
          apply div_pos hδ
          exact (show (0 : NNRat) < 3 by norm_num)
        
        let coe : NNRat → ENNRat := fun r => (r : ENNRat)
        
        have h1_lt : i1 < coe (r1 + ε) := by
          rw [← hr1]; exact WithTop.coe_lt_coe.mpr (lt_add_of_pos_right r1 hε_pos)
        have h2_lt : i2 < coe (r2 + ε) := by
          rw [← hr2]; exact WithTop.coe_lt_coe.mpr (lt_add_of_pos_right r2 hε_pos)
        
        obtain ⟨x, hx, hx_lt⟩ := h1.exists_lt_of_lt h1_lt
        obtain ⟨y, hy, hy_lt⟩ := h2.exists_lt_of_lt h2_lt
        
        have hxy_lt : x + y < coe (r1 + ε) + coe (r2 + ε) := add_lt_add hx_lt hy_lt
        have h_bound : coe (r1 + ε) + coe (r2 + ε) ≤ coe rk := by
          rw [← WithTop.coe_add, WithTop.coe_le_coe]
          have : 2 * ε ≤ δ := by
             rw [mul_comm]
             apply (le_div_iff (show (0 : NNRat) < 3 by norm_num)).mpr
             apply mul_le_mul_of_nonneg_left (by norm_num) hδ.le
          calc r1 + ε + (r2 + ε)
            _ = r1 + r2 + (ε + ε) := by
              rw [add_assoc r1 ε (r2 + ε), ← add_assoc ε r2 ε, add_comm ε r2, add_assoc r2 ε ε, ← add_assoc r1 r2 (ε + ε)]
            _ = r1 + r2 + 2 * ε := by rw [two_mul]
            _ ≤ r1 + r2 + δ := add_le_add_left this (r1 + r2)
            _ = r1 + r2 + (rk - (r1 + r2)) := rfl
            _ = rk := add_tsub_cancel_of_le h_r12_rk.le
        
        have h_lt_k : x + y < k := by
          rw [← hrk]
          exact hxy_lt.trans_le h_bound
        exact not_le_of_lt h_lt_k (hk (x + y) ⟨x, hx, y, hy, rfl⟩)

end Coh.Boundary
