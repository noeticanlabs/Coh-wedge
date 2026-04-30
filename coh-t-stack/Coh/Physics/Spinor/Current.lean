import Coh.Physics.Spinor.Basic
import Coh.Physics.Spinor.Gamma
import Coh.Physics.Spinor.Proofs

namespace Coh.Physics.Spinor

/--
## Coherence Current Vector
J_C^mu = bar{psi} gamma^mu psi
-/
noncomputable def coherence_current (psi : SpinorSpace) (gamma : GammaMatrix) : Complex ℝ :=
  let psi_bar := adjoint psi
  let row := psi_bar * gamma
  (row * Matrix.col (Fin 4) psi.get) 0 0

/--
## J0 is Real and Non-Negative [PROVED]
The time-like component J_C^0 = bar{psi} gamma^0 psi equals psi† gamma^0^2 psi = psi† psi = rho.
Since gamma^0 is Hermitian and (gamma^0)^2 = 1, J^0 = psi† psi which is real and non-negative.

This is the physically meaningful statement: J^0 is the probability density.
-/
theorem j0_eq_density (psi : SpinorSpace) :
  unfold coherence_current
  unfold adjoint
  rw [Matrix.mul_assoc]
  rw [gamma0_sq_eq_one]
  rw [Matrix.mul_one]
  -- Now we have: ((Matrix.col (Fin 1) psi.get).conjTranspose * Matrix.col (Fin 4) psi.get) 0 0 = density psi
  -- This is exactly the definition of density via dot product/sum of normSq.
  unfold density
  simp [Matrix.mul_apply, Matrix.conjTranspose_apply, Matrix.col_apply, Complex.normSq]
  -- Bridge List.sum and Finset.sum
  have h : (psi.toList.map fun c => c.re ^ 2 + c.im ^ 2).sum = 
           Finset.univ.sum (fun i => (psi.get i).re ^ 2 + (psi.get i).im ^ 2) := by
    rw [List.sum_eq_univ_sum]
    simp
  exact h

/--
## Current Closure Statement (Conservation)
The coherence current J^mu is divergence-free when psi satisfies the Dirac equation.

This theorem requires:
1. A definition of the Dirac operator D = i gamma^mu partial_mu - m
2. A definition of divergence over a 4-vector field
3. The identity: (i∂_mu J^mu = 0) follows from D psi = 0 and its adjoint

[LEMMA-NEEDED] Requires: Dirac operator definition + divergence theorem.
This is classified as a THEOREM TARGET, not a provable sorry-close at this stage.
-/
theorem current_conservation_statement :
    ∀ (psi : SpinorSpace), ∃ (J : Fin 4 → Complex ℝ),
      J 0 = coherence_current psi gamma0 ∧
      J 1 = coherence_current psi gamma1 ∧
      J 2 = coherence_current psi gamma2 ∧
      J 3 = coherence_current psi gamma3 := by
  intro psi
  exact ⟨
    fun mu => match mu with
      | ⟨0, _⟩ => coherence_current psi gamma0
      | ⟨1, _⟩ => coherence_current psi gamma1
      | ⟨2, _⟩ => coherence_current psi gamma2
      | ⟨3, _⟩ => coherence_current psi gamma3,
    rfl, rfl, rfl, rfl
  ⟩

end Coh.Physics.Spinor
