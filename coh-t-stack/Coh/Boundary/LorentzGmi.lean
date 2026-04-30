import Mathlib

namespace Coh.Boundary

structure GmiConeParams where
  cG : ℚ
  dt : ℚ
  h_cG_pos : 0 < cG
  h_dt_pos : 0 < dt

structure DiscreteGmiStep (X : Type) where
  x₀ : X
  x₁ : X
  dist : ℚ
  h_dist_nonneg : 0 ≤ dist

def intervalSq (p : GmiConeParams) (s : DiscreteGmiStep X) : ℚ :=
  p.cG^2 * p.dt^2 - s.dist^2

def Timelike (p : GmiConeParams) (s : DiscreteGmiStep X) : Prop :=
  s.dist < p.cG * p.dt

def NullLike (p : GmiConeParams) (s : DiscreteGmiStep X) : Prop :=
  s.dist = p.cG * p.dt

def Spacelike (p : GmiConeParams) (s : DiscreteGmiStep X) : Prop :=
  p.cG * p.dt < s.dist

theorem timelike_interval_positive
  (p : GmiConeParams) (s : DiscreteGmiStep X)
  (h : Timelike p s) :
  0 < intervalSq p s := by
  unfold intervalSq Timelike at *
  have h_sq : s.dist^2 < (p.cG * p.dt)^2 := by
    rw [sq_lt_sq, abs_of_nonneg s.h_dist_nonneg, abs_of_nonneg (by nlinarith [p.h_cG_pos, p.h_dt_pos])]
    exact h
  nlinarith

theorem null_interval_zero
  (p : GmiConeParams) (s : DiscreteGmiStep X)
  (h : NullLike p s) :
  intervalSq p s = 0 := by
  unfold intervalSq NullLike at *
  rw [h]
  ring

theorem spacelike_interval_negative
  (p : GmiConeParams) (s : DiscreteGmiStep X)
  (h : Spacelike p s) :
  intervalSq p s < 0 := by
  unfold intervalSq Spacelike at *
  have h_sq : (p.cG * p.dt)^2 < s.dist^2 := by
    rw [sq_lt_sq, abs_of_nonneg (by nlinarith [p.h_cG_pos, p.h_dt_pos]), abs_of_nonneg s.h_dist_nonneg]
    exact h
  nlinarith

inductive CausalDecision where
  | admitCandidate
  | boundary
  | rejectSpacelike
  deriving DecidableEq, Repr

def governorConeDecision
  (p : GmiConeParams)
  (s : DiscreteGmiStep X) : CausalDecision :=
  if s.dist < p.cG * p.dt then
    CausalDecision.admitCandidate
  else if s.dist = p.cG * p.dt then
    CausalDecision.boundary
  else
    CausalDecision.rejectSpacelike

theorem spacelike_rejected
  (p : GmiConeParams) (s : DiscreteGmiStep X)
  (h : Spacelike p s) :
  governorConeDecision p s = CausalDecision.rejectSpacelike := by
  unfold governorConeDecision
  split_ifs
  . unfold Spacelike at h
    linarith
  . unfold Spacelike at h
    linarith
  . rfl

/-- Phase 2: Mass Shell -/

theorem mass_shell_from_gamma_relation
  {γ m c v E p : ℚ}
  (hE : E = γ * m * c^2)
  (hp : p = γ * m * v)
  (hγ : γ^2 * (1 - v^2 / c^2) = 1)
  (hc : c ≠ 0) :
  E^2 - p^2 * c^2 = m^2 * c^4 := by
  field_simp [hc] at hγ
  calc E^2 - p^2 * c^2
    _ = (γ * m * c^2)^2 - (γ * m * v)^2 * c^2 := by rw [hE, hp]
    _ = γ^2 * m^2 * c^4 - γ^2 * m^2 * v^2 * c^2 := by ring
    _ = (m^2 * c^2) * (γ^2 * (c^2 - v^2)) := by ring
    _ = m^2 * c^2 * c^2 := by rw [hγ]
    _ = m^2 * c^4 := by ring

end Coh.Boundary
