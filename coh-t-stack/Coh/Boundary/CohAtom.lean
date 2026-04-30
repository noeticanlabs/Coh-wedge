import Mathlib
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## Coh Atom Framework
\boxed{ \textbf{Coh Atom}=\text{a self-contained, verifier-governed unit of state that generates, filters, and executes CohBits.} }
-/

/--
### Coh Atom Definition
\boxed{\mathcal A(x) = (x, \mathcal B_x, \mathcal A_x, \mathcal P_x, \mathcal G_x, \mathcal M_x, \mathcal R_x)}
-/
structure CohAtom {X : Type} (S : CohSystem X) where
  state : X
  budget : ENNRat
  receipt_chain : List (Type) -- Simplified representation of H_t
  
  -- Candidate bits are just the set of all CohBits at this state
  -- Executable bits are the subset satisfying is_executable
  -- (These are properties derived from the state and system)

/--
### Executable Direction Set
The set of indices i such that b_i is executable.
-/
def executable_set {X : Type} {S : CohSystem X} (a : CohAtom S) (bits : List (CohBit S a.state)) : List (CohBit S a.state) :=
  bits.filter (fun b => is_executable b)

/--
### Transition Law
\boxed{\mathcal A(x) \xrightarrow{\mathfrak b_i} \mathcal A(x_i')}
-/
def transition {X : Type} {S : CohSystem X} (a : CohAtom S) (b : CohBit S a.state) (h_exec : is_executable b) (refresh : ENNRat) : CohAtom S where
  state := b.next_state
  budget := a.budget + refresh - S.spend b.transition
  receipt_chain := b.certificate :: a.receipt_chain

/--
### Metabolic Law
Constraint: V(x') + Spend(e) ≤ V(x) + Defect(e) + A_t
-/
def metabolic_admissible {X : Type} {S : CohSystem X} (a : CohAtom S) (b : CohBit S a.state) (refresh : ENNRat) : Prop :=
  S.valuation b.next_state + S.spend b.transition ≤ S.valuation a.state + b.defect + refresh

/--
### Identity Atom Law
The identity transition preserves the atom's state and valuation.
-/
theorem identity_atom_stable {X : Type} {S : CohSystem X} (a : CohAtom S) (cx : Type) (h_rv : S.rv_accept cx) :
  let b_id := identity_cohbit S a.state cx h_rv
  let a_next := transition a b_id (by 
    unfold is_executable
    simp [identity_cohbit, h_rv]
  ) 0
  a_next.state = a.state := by
  simp [transition, identity_cohbit]

/--
### Protected Identity Theorem
If refresh is 0 and no executable bits exist except identity, the atom enters stasis.
-/
def is_protected {X : Type} {S : CohSystem X} (a : CohAtom S) (cx : Type) (h_rv : S.rv_accept cx) : Prop :=
  ∀ (b : CohBit S a.state), is_executable b -> b.transition = id

end Coh.Boundary
