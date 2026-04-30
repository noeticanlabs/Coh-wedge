import Mathlib.Data.Matrix.Basic
import Mathlib.Analysis.Complex.Basic
import Coh.Physics.Spinor.Basic

namespace Coh.Physics.Spinor

/--
## Gamma Matrix
4x4 complex matrix representation for Dirac carriers.
-/
def GammaMatrix := Matrix (Fin 4) (Fin 4) (Complex ℝ)

/--
## Gamma 0 (Dirac Representation)
[[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, -1, 0], [0, 0, 0, -1]]
-/
def gamma0 : GammaMatrix := !![
  1, 0, 0, 0;
  0, 1, 0, 0;
  0, 0, -1, 0;
  0, 0, 0, -1
]

/--
## Gamma 1 (Dirac Representation)
[[0, 0, 0, 1], [0, 0, 1, 0], [0, -1, 0, 0], [-1, 0, 0, 0]]
-/
def gamma1 : GammaMatrix := !![
  0, 0, 0, 1;
  0, 0, 1, 0;
  0, -1, 0, 0;
  -1, 0, 0, 0
]

-- ... gamma2, gamma3 would follow similarly

/--
## Dirac Adjoint
psi_bar = psi† gamma0
-/
noncomputable def adjoint (psi : SpinorSpace) : Matrix (Fin 1) (Fin 4) (Complex ℝ) :=
  (Matrix.row (Fin 1) psi.get).conjTranspose * gamma0

end Coh.Physics.Spinor
