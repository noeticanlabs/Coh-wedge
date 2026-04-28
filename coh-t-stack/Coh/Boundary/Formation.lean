import Coh.Boundary.LawOfChaos

namespace Coh.Boundary

/--
A Coherence Object formalizes backward executable justification (the Verifier).
-/
structure CoherenceObject (X Q S : Type) [OrderedAddCommMonoid S] where
  RV : X → Q → X → Prop
  V : X → S
  Spend : Q → S
  Defect : Q → S

def CohAdmissible {X Q S : Type} [OrderedAddCommMonoid S]
  (obj : CoherenceObject X Q S) (x : X) (R : Q) (y : X) : Prop :=
  obj.RV x R y ∧ obj.V y + obj.Spend R ≤ obj.V x + obj.Defect R

/--
Formation is the intersection of forward Chaos generation and backward Coherence justification.
-/
structure ChaosCoherenceSystem (G P R X Q S : Type) 
  [OrderedAddCommMonoid R] [OrderedAddCommMonoid S] where
  Chaos : ChaosObject G P R
  Coh : CoherenceObject X Q S
  Pi : (G × P × G) → (X × Q × X)
  Rho : R → S
  Rho_monotone : Monotone Rho

def FormationAdmissible {G P R X Q S : Type} 
  [OrderedAddCommMonoid R] [OrderedAddCommMonoid S]
  (sys : ChaosCoherenceSystem G P R X Q S) (z : G × P × G) : Prop :=
  let (g, p, g') := z
  ChaosAdmissible sys.Chaos g p g' ∧ 
  let (x, R, y) := sys.Pi z
  CohAdmissible sys.Coh x R y

/--
Theorem: Formation admissibility implies Coherence admissibility.
This proves that the Formation mode is a conservative strengthening of the verifier.
-/
theorem formation_implies_coherence {G P R X Q S : Type} 
  [OrderedAddCommMonoid R] [OrderedAddCommMonoid S]
  (sys : ChaosCoherenceSystem G P R X Q S) (z : G × P × G) :
  FormationAdmissible sys z → CohAdmissible sys.Coh (sys.Pi z).1 (sys.Pi z).2.1 (sys.Pi z).2.2 := by
  intro h
  exact h.right

end Coh.Boundary
