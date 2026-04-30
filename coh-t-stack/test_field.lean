import Mathlib

abbrev ENNRat := WithTop NNRat

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

theorem stressEnergyTensor_symmetric (Psi : MatterField) :
  ∀ mu nu, stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu := by
  intros mu nu
  unfold stressEnergyTensor
  apply mul_comm

theorem field_equation_effective_metric {g : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (hl : l < 1) (h : FieldEquation g Psi kappa l) :
  EffMetric g := by
  constructor
  intro mu nu
  have h1 := h.holds mu nu
  have h2 := h.holds nu mu
  have hT : stressEnergyTensor Psi mu nu = stressEnergyTensor Psi nu mu :=
    stressEnergyTensor_symmetric Psi mu nu
  unfold curvatureTerm at h1 h2
  rw [hT] at h1
  -- Now h1 and h2 show that g mu nu and g nu mu satisfy the same equation.
  -- Since l < 1, the solution is unique.
  sorry

theorem field_equation_unique {g1 g2 : Tensor2} {Psi : MatterField} {kappa l : ENNRat}
  (h1 : FieldEquation g1 Psi kappa l)
  (h2 : FieldEquation g2 Psi kappa l)
  (hl : l < 1) :
  g1 = g2 := by
  funext mu nu
  have e1 := h1.holds mu nu
  have e2 := h2.holds mu nu
  unfold curvatureTerm at e1
  unfold curvatureTerm at e2
  sorry
