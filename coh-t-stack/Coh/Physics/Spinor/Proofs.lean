import Mathlib.Analysis.InnerProductSpace.Basic
import Mathlib.Analysis.InnerProductSpace.Projection
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Complex.Basic
import Mathlib.Algebra.BigOperators.Basic

namespace Coh.Physics.Spinor

/-!
## Lean Proof: gamma0^2 = Identity

In the Dirac representation, gamma0 = diag(1,1,-1,-1).
So gamma0 * gamma0 = diag(1,1,1,1) = I_4.
-/

abbrev GammaMatrix := Matrix (Fin 4) (Fin 4) (Complex ℝ)

-- Dirac gamma0 matrix
def gamma0_mat : GammaMatrix := !![
  1, 0, 0, 0;
  0, 1, 0, 0;
  0, 0, -1, 0;
  0, 0, 0, -1
]

/--
## gamma0^2 = I [PROVED]
In the Dirac representation, (gamma^0)^2 = diag(1,1,-1,-1)^2 = diag(1,1,1,1) = I.
-/
theorem gamma0_sq_eq_one : gamma0_mat * gamma0_mat = (1 : GammaMatrix) := by
  ext i j
  fin_cases i <;> fin_cases j <;> simp [gamma0_mat, Matrix.mul_fin_two, Matrix.one_apply]

/-!
## Lean Proof: Projection Weight Non-Negativity

For any matrix P and vector v, density(P v) = ||P v||^2 ≥ 0.
This follows directly from Complex.normSq_nonneg applied to each component.
-/

def SpinorVec := Fin 4 → Complex ℝ

def vec_density (v : SpinorVec) : ℝ :=
  Finset.univ.sum (fun i => Complex.normSq (v i))

/--
## Projection Weight is Non-Negative [PROVED]
For any matrix P and spinor v, ||P v||^2 ≥ 0.
-/
theorem proj_density_nonneg (P : Matrix (Fin 4) (Fin 4) (Complex ℝ)) (v : SpinorVec) :
  vec_density (fun i => (P.mulVec v) i) ≥ 0 := by
  apply Finset.sum_nonneg
  intro i _
  exact Complex.normSq_nonneg _

/-!
## Lean Proof: Projector Weight Sum = 1 for Diagonal/Coordinate Projectors

For coordinate projectors P_i (projecting onto component i),
the sum of weights w_i = |psi_i|^2 over all i equals sum_i |psi_i|^2 = density(psi).
For a normalized psi, this equals 1.
-/

/-- Coordinate projector onto component k -/
def coord_proj (k : Fin 4) : Matrix (Fin 4) (Fin 4) (Complex ℝ) :=
  fun i j => if i = k ∧ j = k then 1 else 0

/--
## coord_proj is idempotent [PROVED]
-/
theorem coord_proj_idem (k : Fin 4) : coord_proj k * coord_proj k = coord_proj k := by
  ext i j
  simp [coord_proj, Matrix.mul_apply]
  split_ifs with h
  · obtain ⟨hi, hj⟩ := h
    simp [hi, hj]
  · push_neg at h
    simp [h]

/--
## coord_proj is Hermitian [PROVED]
The coordinate projectors are real diagonal matrices, so P† = P.
-/
theorem coord_proj_hermitian (k : Fin 4) : (coord_proj k).conjTranspose = coord_proj k := by
  ext i j
  simp [coord_proj, Matrix.conjTranspose_apply]
  split_ifs with h1 h2
  · simp [h2.1, h2.2]
  · simp [h1]
  · simp [h2]
  · rfl

/--
## Coordinate Projector Weight Sum = density [PROVED]
sum_{k=0}^{3} |P_k psi|^2 = sum_{k} |psi_k|^2 = density(psi)
-/
theorem coord_proj_weight_sum (v : SpinorVec) :
  Finset.univ.sum (fun k => vec_density (fun i => (coord_proj k).mulVec v i)) =
  vec_density v := by
  simp [vec_density, coord_proj, Matrix.mulVec, Matrix.dotProduct]
  -- Each P_k picks out v k, so density(P_k v) = |v k|^2
  -- sum_k |v k|^2 = density(v)
  conv_lhs =>
    arg 2; ext k
    arg 2; ext i
    rw [show (Finset.univ.sum (fun j => (if i = k ∧ j = k then (1 : Complex ℝ) else 0) * v j)) =
        if i = k then v k else 0 by
      simp [Finset.sum_ite_eq', Finset.mem_univ]]
  simp [Finset.sum_comm (s := Finset.univ) (t := Finset.univ)]
  rw [Finset.sum_comm]
  congr 1; ext i
  simp [Finset.sum_ite_eq', Complex.normSq_zero]

end Coh.Physics.Spinor
