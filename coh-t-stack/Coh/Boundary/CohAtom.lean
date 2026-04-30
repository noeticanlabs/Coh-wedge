import Mathlib
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## Coh Atom
The smallest bound system that can generate, verify, receipt, and continue CohBits.
-/
structure CohAtom (X Y : Type) where
  /-- Hidden/proposal/microstate space -/
  space_X : Type := X
  /-- Visible record/claim/macroscopic state space -/
  space_Y : Type := Y
  /-- Projection from possibility to record -/
  projection : X → Y
  
  /-- NPE: Proposal Kernel -/
  npe : X → Set X
  /-- RV: Verifier Nucleus -/
  rv : Y → Prop
  /-- PhaseLoom: Memory-Feedback Kernel -/
  phaseloom : Y → Set (X → Set X)
  
  /-- Global Governance Boundary -/
  governor : X → Prop
  
  /-- Receipt Ledger -/
  receipt : Y → Type
  
  /-- Hierarchical Budget Envelope -/
  budget : X → ℝ
  
  /-- Feedback map from receipts to PhaseLoom/NPE -/
  feedback : (y : Y) → receipt y → (X → Set X)

/--
## Atom Stability Law
A Coh Atom is stable if the global Coherence Law holds for its transitions.
-/
def is_stable {X Y : Type} (atom : CohAtom X Y) (x_prev x_next : X) (a : Y) : Prop :=
  atom.governor x_next ∧ 
  atom.budget x_next + 10 ≤ atom.budget x_prev + 100

/--
## Atomic Transition (CohBit Emission)
The process by which a Coh Atom emits a CohBit.

Note: We parameterize over the receipt explicitly to avoid the sorry.
The receipt `r : atom.receipt y` is a required precondition for the feedback
continuation — it cannot be synthesized without an actual RV decision.
The caller must supply it. The definition is existentially quantified over
all admissible receipts.
-/
def atomic_transition {X Y : Type} (atom : CohAtom X Y) (x : X) : Set (Y × X) :=
  { p | 
    let y := p.1
    let x_next := p.2
    ∃ x_proposal ∈ atom.npe x, 
    y = atom.projection x_proposal ∧ 
    atom.rv y ∧ 
    (∃ r : atom.receipt y, x_next ∈ (atom.feedback y r) x_proposal) ∧
    is_stable atom x x_next y
  }

/--
## Atomic Transition Stability
Every completed atomic transition satisfies the Stability Law.
[PROVED]
-/
theorem atomic_transition_stable {X Y : Type} (atom : CohAtom X Y) (x : X)
  (p : Y × X) (hp : p ∈ atomic_transition atom x) :
  is_stable atom x p.2 p.1 := by
  obtain ⟨_, _, _, _, _, hstable⟩ := hp
  exact hstable

/--
## Atomic Transition Verified
Every completed atomic transition passes the RV gate.
[PROVED]
-/
theorem atomic_transition_rv_certified {X Y : Type} (atom : CohAtom X Y) (x : X)
  (p : Y × X) (hp : p ∈ atomic_transition atom x) :
  atom.rv p.1 := by
  obtain ⟨_, _, _, hrv, _, _⟩ := hp
  exact hrv

end Coh.Boundary
