import Mathlib
import Coh.Boundary.RationalInf

namespace Coh.Boundary

/--
## CohBit Framework
\boxed{ \textbf{CohBit}=\text{a certified, projection-aware, probability-weighted, verifier-gated transition atom.} }
-/

/--
### Base Coh System
Defines the state space and metric properties for Coherence.
-/
structure CohSystem (X : Type) where
  valuation : X -> ENNRat
  projection : (X -> X) -> (X -> X) -- Hidden to Observable trace mapping
  certified_defect : (X -> X) -> ENNRat -- \widehat{\delta}
  spend : (X -> X) -> ENNRat
  rv_accept : Type -> Prop -- Runtime Verifier
  id_defect_zero : certified_defect id = 0

/--
### CohBit Definition
\boxed{\mathfrak b_i(x) = (r_i, x_i', R_i, m_i, u_i, p_i, c_i)}
-/
structure CohBit {X : Type} (S : CohSystem X) (x : X) where
  transition : X -> X
  next_state : X
  observable_trace : X -> X
  certificate : Type
  utility : ENNRat
  
  -- Constraints
  trace_eq : observable_trace = S.projection transition
  next_eq : next_state = transition x
  
  -- Admissibility Margin: m_i(x) = V(x) + D_i(x) - V(x_i') - Spend(r_i)
  -- Note: Using a signed representation or inequality for Lean.
  -- Here we define D_i and require it satisfies the V2 condition.
  defect : ENNRat
  v2_certified : defect ≥ S.certified_defect transition

/--
### Admissibility Condition
A CohBit is executable iff m_i(x) ≥ 0 and RV(c_i) = ACCEPT.
-/
def is_executable {X : Type} {S : CohSystem X} {x : X} (b : CohBit S x) : Prop :=
  (S.valuation x + b.defect ≥ S.valuation b.next_state + S.spend b.transition) ∧ 
  (S.rv_accept b.certificate)

/--
### Composition Law
If b_i and b_j are executable and certificates compose, then the composition is executable.
-/
theorem composition_stability {X : Type} {S : CohSystem X} {x y z : X} 
  (bi : CohBit S x) (bj : CohBit S y)
  (h_next : bi.next_state = y)
  (h_exec_i : is_executable bi)
  (h_exec_j : is_executable bj) :
  -- Composite Admissibility Margin
  S.valuation x + (bi.defect + bj.defect) ≥ S.valuation bj.next_state + (S.spend bi.transition + S.spend bj.transition) := by
  have hi := h_exec_i.1
  have hj := h_exec_j.1
  rw [h_next] at hi
  -- V(x) + Di ≥ V(y) + Si
  -- V(y) + Dj ≥ V(z) + Sj
  -- V(x) + Di + Dj ≥ V(z) + Si + Sj
  calc
    S.valuation x + (bi.defect + bj.defect) 
      = (S.valuation x + bi.defect) + bj.defect := by rw [add_assoc]
    _ ≥ (S.valuation y + S.spend bi.transition) + bj.defect := add_le_add_right hi _
    _ = S.spend bi.transition + (S.valuation y + bj.defect) := by rw [add_comm, add_assoc, add_comm bj.defect]
    _ ≥ S.spend bi.transition + (S.valuation bj.next_state + S.spend bj.transition) := add_le_add_left hj _
    _ = S.valuation bj.next_state + (S.spend bi.transition + S.spend bj.transition) := by rw [add_assoc, add_comm (S.spend bi.transition)]

/--
### Identity CohBit
\boxed{ \mathbf 1_x = (\mathrm{id}_x, x, \Pi(\mathrm{id}_x), 0, u_0, 1, c_x) }
-/
def identity_cohbit {X : Type} (S : CohSystem X) (x : X) (cx : Type) (h_rv : S.rv_accept cx) : CohBit S x where
  transition := id
  next_state := x
  observable_trace := S.projection id
  certificate := cx
  utility := 0
  trace_eq := rfl
  next_eq := rfl
  defect := 0
  v2_certified := by 
    rw [S.id_defect_zero]
    exact le_refl 0

end Coh.Boundary
