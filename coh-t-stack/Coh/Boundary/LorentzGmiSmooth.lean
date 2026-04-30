import Mathlib.Analysis.SpecialFunctions.Log.Basic
import Mathlib.Analysis.SpecialFunctions.Exp
import Mathlib.Data.Real.Sqrt

namespace Coh.Boundary

noncomputable section

/-- GMI smooth velocity barrier potential -/
def barrierPotential (v c : ℝ) : ℝ :=
  -(1/2) * Real.log (1 - v^2 / c^2)

/-- Viability clock principle: intrinsic time dilation from barrier -/
theorem viability_dilation_from_barrier
  {v c : ℝ}
  (hc : 0 < c)
  (hv : v^2 < c^2) :
  Real.exp (-(barrierPotential v c)) = Real.sqrt (1 - v^2 / c^2) := by
  unfold barrierPotential
  simp
  -- exp(1/2 * log(x)) = sqrt(x)
  have h_inner : 0 < 1 - v^2 / c^2 := by
    have : v^2 / c^2 < 1 := (div_lt_one (sq_pos_of_ne_zero _ (by linarith))).mpr hv
    linarith
  rw [mul_comm, ← div_eq_mul_inv, ← Real.log_sqrt (le_of_lt h_inner)]
  rw [Real.exp_log]
  exact Real.sqrt_pos.mpr h_inner

/-- The GMI Lorentz factor -/
def gammaFactor (v c : ℝ) : ℝ :=
  (Real.sqrt (1 - v^2 / c^2))⁻¹

/-- Energy-Momentum definitions in smooth GMI -/
def gmiEnergy (m γ c : ℝ) : ℝ := γ * m * c^2
def gmiMomentum (m γ v : ℝ) : ℝ := γ * m * v

/-- Continuous Mass Shell Theorem -/
theorem mass_shell_smooth
  {v c m : ℝ}
  (hc : 0 < c)
  (hv : v^2 < c^2)
  (m_pos : 0 < m) :
  let γ := gammaFactor v c
  (gmiEnergy m γ c)^2 - (gmiMomentum m γ v)^2 * c^2 = m^2 * c^4 := by
  intro γ
  have h_inner : 0 < 1 - v^2 / c^2 := by
    have : v^2 / c^2 < 1 := (div_lt_one (sq_pos_of_ne_zero _ (by linarith))).mpr hv
    linarith
  have h_γ_sq_inv : γ^2 * (1 - v^2 / c^2) = 1 := by
    dsimp [γ]
    unfold gammaFactor
    rw [inv_pow, Real.sq_sqrt (le_of_lt h_inner)]
    have h_c2 : c^2 ≠ 0 := by nlinarith
    have h_cv : c^2 - v^2 ≠ 0 := by nlinarith
    field_simp [h_c2, h_cv]
  
  unfold gmiEnergy gmiMomentum
  calc (γ * m * c^2)^2 - (γ * m * v)^2 * c^2
    _ = γ^2 * m^2 * c^2 * (c^2 - v^2) := by ring
    _ = γ^2 * (1 - v^2 / c^2) * m^2 * c^4 := by 
        have h_c2 : c^2 ≠ 0 := by nlinarith
        field_simp [h_c2]
        ring
    _ = 1 * m^2 * c^4 := by rw [h_γ_sq_inv]
    _ = m^2 * c^4 := by ring

end 

end Coh.Boundary
