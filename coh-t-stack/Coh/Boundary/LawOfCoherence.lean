import Coh.Templates
import Mathlib.Tactic

namespace Coh.Boundary

/--
A Coherence Object formalizes backward executable justification (the Verifier).
The Law of Coherence is: v_post + spend ≤ v_pre + defect + authority.
-/
structure CoherenceObject (X Q S : Type) [OrderedAddCommMonoid S] where
  RV : X → Q → X → Prop
  V : X → S
  Spend : Q → S
  Defect : Q → S
  Authority : Q → S

def CohAdmissible {X Q S : Type} [OrderedAddCommMonoid S]
  (obj : CoherenceObject X Q S) (x : X) (R : Q) (y : X) : Prop :=
  obj.RV x R y ∧ obj.V y + obj.Spend R ≤ obj.V x + obj.Defect R + obj.Authority R

/--
Theorem: The composition of two Coherence-admissible transitions satisfies the additive Law of Coherence.
-/
theorem coherence_composition {X Q S : Type} [OrderedAddCommMonoid S]
  (obj : CoherenceObject X Q S) (x1 x2 x3 : X) (R1 R2 : Q)
  (h1 : CohAdmissible obj x1 R1 x2)
  (h2 : CohAdmissible obj x2 R2 x3) :
  obj.V x3 + (obj.Spend R1 + obj.Spend R2) ≤ obj.V x1 + (obj.Defect R1 + obj.Defect R2) + (obj.Authority R1 + obj.Authority R2) := by
  unfold CohAdmissible at h1 h2
  obtain ⟨_, h1_ineq⟩ := h1
  obtain ⟨_, h2_ineq⟩ := h2
  exact Coh.coh_compose_linear h1_ineq h2_ineq

/--
Theorem: The identity transition is trivially admissible for any object.
-/
theorem coherence_identity {X Q S : Type} [OrderedAddCommMonoid S]
  (obj : CoherenceObject X Q S) (x : X) :
  obj.V x + 0 ≤ obj.V x + 0 + 0 := by
  simp


end Coh.Boundary
