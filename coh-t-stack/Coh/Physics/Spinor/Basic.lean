import Mathlib.Analysis.Complex.Basic
import Mathlib.Data.Vector
import Mathlib.Analysis.InnerProductSpace.Basic
import Mathlib.Data.Matrix.Basic

namespace Coh.Physics.Spinor

/--
## Coh Spinor Hilbert Space
A 4-dimensional complex vector space for Dirac-like carriers.
-/
def SpinorSpace := Vector (Complex ℝ) 4

/--
## Coh Spinor State
The internal carrier state for a Coh Atom.
-/
structure CohSpinor where
  state : SpinorSpace
  /-- Probability density must be normalized for a bound atom carrier -/
  normalized : (state.toList.map (fun c => Complex.normSq c)).sum = 1

/--
## Spinor Density
The detector-frame pre-measure density (rho_C = psi_C† psi_C).
-/
def density (psi : SpinorSpace) : ℝ :=
  (psi.toList.map (fun c => Complex.normSq c)).sum

/--
## Positive Density Theorem
The probability density of a Coh Spinor is always non-negative.
-/
theorem positive_density_theorem (psi : SpinorSpace) : density psi ≥ 0 := by
  unfold density
  apply List.sum_nonneg
  intro c h
  simp
  apply Complex.normSq_nonneg

/--
## Orthogonal Projector
Represented as a 4x4 complex matrix.
-/
def Projector := Matrix (Fin 4) (Fin 4) (Complex ℝ)

def is_projector (P : Projector) : Prop :=
  P * P = P ∧ P.conjTranspose = P

/--
## Projection Weight
w_i = |P_i psi|^2
-/
noncomputable def projection_weight (P : Projector) (psi : SpinorSpace) : ℝ :=
  let projected := P * Matrix.col (Fin 4) psi.get
  density (Vector.ofFn (fun i => projected i 0))

/--
## Projection Weight Theorem
For a complete set of orthogonal projectors, the weights sum to the total density.
-/
theorem projection_weight_theorem (psi : CohSpinor) (Ps : List Projector) 
  (h_proj : ∀ P ∈ Ps, is_projector P) 
  (h_sum : Ps.sum = Matrix.one (Fin 4) (Complex ℝ)) :
  (Ps.map (fun P => projection_weight P psi.state)).sum = 1 := by
  -- Proof sketch: sum of projected densities equals density of sum of projections
  sorry

end Coh.Physics.Spinor
