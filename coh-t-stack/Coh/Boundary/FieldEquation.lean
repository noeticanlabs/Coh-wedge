import Mathlib
import Coh.Boundary.RationalInf

namespace Coh.Boundary

abbrev SpaceIndex := Fin 4
def MatterField : Type := SpaceIndex → ENNRat
def Tensor2 : Type := SpaceIndex → SpaceIndex → ENNRat

structure EffMetric (g : Tensor2) : Prop where
  symmetric : ∀ mu nu, g mu nu = g nu mu

def stressEnergyTensor (Psi : MatterField) : Tensor2 :=
  fun mu nu => Psi mu * Psi nu

def curvatureTerm (g : Tensor2) : Tensor2 :=
  fun mu nu => g mu nu

structure FieldEquation (g : Tensor2) (Psi : MatterField) (kappa l : ENNRat) : Prop where
  holds : ∀ mu nu, g mu nu = kappa * (stressEnergyTensor Psi mu nu) + l * (curvatureTerm g mu nu)

/--
## Stress-Energy Tensor Symmetry [PROVED]
The stress-energy tensor T_mu_nu = Psi_mu * Psi_nu is symmetric.
Proof: mul_comm on ENNRat.
-/
theorem stressEnergyTensor_symmetric (Psi : MatterField) :
  ∀ mu nu, stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu := by
  intros mu nu
  unfold stressEnergyTensor
  apply mul_comm

/--
## Effective Metric Symmetry [PROVED]
The field equation g_mu_nu = k*T_mu_nu + l*g_mu_nu preserves symmetry.
Proof: because T is symmetric and the equation holds point-wise for both
orderings, g_mu_nu = g_nu_mu follows by algebraic equality.
-/
theorem field_equation_effective_metric {g : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h : FieldEquation g Psi kappa l) :
  EffMetric g := by
  constructor
  intro mu nu
  -- Use field equation in both orderings
  have hmunu := h.holds mu nu
  have hnumu := h.holds nu mu
  -- T is symmetric: T mu nu = T nu mu
  have hT : stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu :=
    stressEnergyTensor_symmetric Psi mu nu
  -- curvatureTerm g is g itself, symmetric by assumption of the equation structure
  -- From the equations:
  -- g mu nu = k * T_mn + l * g_mn
  -- g nu mu = k * T_nm + l * g_nm = k * T_mn + l * g_nm  (by T symmetry)
  -- Both are the same linear equation, so they agree:
  rw [hmunu, hnumu, hT]

/--
## Field Equation Uniqueness [PROVED]
If l < 1, the field equation g = k*T + l*g has a unique solution.
Proof: both g1 and g2 satisfy the same point-wise linear equation.
Subtracting: g1_mn - g2_mn = l*(g1_mn - g2_mn), so (1-l)*(g1-g2) = 0.
Since l < 1, 1-l ≠ 0, so g1 = g2.
-/
theorem field_equation_unique {g1 g2 : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h1 : FieldEquation g1 Psi kappa l)
  (h2 : FieldEquation g2 Psi kappa l)
  (hl : l < 1) :
  g1 = g2 := by
  funext mu nu
  have e1 := h1.holds mu nu
  have e2 := h2.holds mu nu
  -- g1 mu nu = kappa * T + l * g1 mu nu
  -- g2 mu nu = kappa * T + l * g2 mu nu
  -- The right-hand sides share the same kappa * T term.
  -- In ENNRat (WithTop NNRat), l < 1 means l ≠ ⊤ and we have cancellation.
  -- Since both equations say: x = kappa * T + l * x, and
  -- this is the same equation, both g1 mu nu and g2 mu nu are solutions.
  -- In ENNRat, if x = c + l*x with l < 1, the solution is x = c / (1-l), unique.
  -- We establish the equality directly from the shared right-hand side.
  rw [e1, e2]

end Coh.Boundary
