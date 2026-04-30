import Mathlib
import Coh.Boundary.RationalInf

namespace Coh.Boundary

abbrev SpaceIndex := Fin 4
def MatterField : Type := SpaceIndex -> ENNRat
def Tensor2 : Type := SpaceIndex -> SpaceIndex -> ENNRat

structure EffMetric (g : Tensor2) : Prop where
  symmetric : forall mu nu, g mu nu = g nu mu

def stressEnergyTensor (Psi : MatterField) : Tensor2 :=
  fun mu nu => Psi mu * Psi nu

def curvatureTerm (g : Tensor2) : Tensor2 :=
  fun mu nu => g mu nu

structure FieldEquation (g : Tensor2) (Psi : MatterField) (kappa l : ENNRat) : Prop where
  holds : forall mu nu, g mu nu = kappa * stressEnergyTensor Psi mu nu + l * curvatureTerm g mu nu

theorem field_equation_unique {g1 g2 : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h1 : FieldEquation g1 Psi kappa l)
  (h2 : FieldEquation g2 Psi kappa l)
  (hl : l < 1) :
  g1 = g2 := by
  sorry

theorem stressEnergyTensor_symmetric (Psi : MatterField) :
  forall mu nu, stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu := by
  intros mu nu
  unfold stressEnergyTensor
  apply mul_comm

theorem field_equation_effective_metric {g : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h : FieldEquation g Psi kappa l) :
  EffMetric g := by
  constructor
  intros mu nu
  sorry

end Coh.Boundary
