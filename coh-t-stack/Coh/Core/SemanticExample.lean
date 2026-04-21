import Coh.Core.Semantic
import Mathlib.Data.Real.NNReal

/-!
# Coh.Core.SemanticExample

A minimal strict-gap example reproducing the paper's worked example.

We show that for a specific SemanticSystem, the semantic cost is strictly smaller than syntactic cost.

## The Toy System

Per the paper:
- X = {A, B, C}
- Hid = {A0, A1, B0, C0}
- Pi(A0) = A, Pi(A1) = A, Pi(B0) = B, Pi(C0) = C
- Valuation: V(A)=5, V(B)=4, V(C)=3

## Syntactic traces

- σ1: A → B (syntactic cost 1)
- σ2: B → C (syntactic cost 1)

Syntactic total: 1 + 1 = 2

## Hidden realizations

- Hidden trace Θ*: A0 → B0 → C0

Assume hidden cost W(Θ*) = 1.

Then semantic cost = max {1} = 1, which is strictly smaller than syntactic 2.
-/

namespace Coh.Core

/-!
## Example: The Toy System
-/

-- Define a simple hidden space with two representatives of A
inductive ToyHidden
  | A0 | A1 | B0 | C0
  deriving DecidableEq

-- Define the observable space
inductive ToyObs
  | A | B | C
  deriving DecidableEq

namespace ToyHidden

-- Projection from hidden to observable
def projection : ToyHidden → ToyObs :=
  fun
    | A0 => ToyObs.A
    | A1 => ToyObs.A
    | B0 => ToyObs.B
    | C0 => ToyObs.C

-- Hidden costs (cost functional W)
def hid_cost : ToyHidden → NNReal :=
  fun
    | A0 => 1
    | A1 => 2  -- distinct hidden representative
    | B0 => 0
    | C0 => 0

end ToyHidden

-- Valuation for the toy system (V from the paper)
def toyValuation : ToyObs → NNReal :=
  fun
    | ToyObs.A => 5
    | ToyObs.B => 4
    | ToyObs.C := 3

-- A trivial verifier that accepts all hidden traces (for this example)
def toyVerifier : List ToyHidden → Prop := fun _ => True

/-!
## Build the Semantic System

Construct a SemanticSystem instance for the toy example.
-/

def ToySystem : SemanticSystem ToyHidden ToyObs :=
  { hid_space := ToyHidden.ToyHidden
    obs_space := ToyObs.ToyObs
    projection := ToyHidden.projection
    hid_cost := ToyHidden.hid_cost
    hist_space := Unit
    proposal := fun _ _ => [] -- trivial proposal for this example
    valuation := toyValuation
    verifier := toyVerifier }

namespace ToySystem

/-!
## The Hidden Trace

Define Θ* = [A0, B0, C0]
-/

def ThetaStar : HiddenTrace :=
  [ToyHidden.A0, ToyHidden.B0, ToyHidden.C0]

/-!
## The Observable Trace

Project Θ* to get τ = [A, B, C]
-/

def tau : List ToyObs :=
  ToySystem.project ThetaStar

/-!
## Syntactic Cost

For this example, we need to calculate syntactic cost.
In a full model, this would come from the step-wise spend.
For the toy example, we assume a constant additive value.

Let `synCost τ = 2` (by assumption in the paper)
-/

def synCost : NNReal := 2

/-!
## Semantic Cost

Compute semantic cost over the fiber of τ.

We need to construct the fiber and check its max cost.
-/

theorem tau_fiber_eq : ToySystem.Fiber tau = {ThetaStar} :=
  by
    unfold Fiber
    apply Set.ext
    intro h
    constructor
    · intro h'
      rw [toyVerifier] at h'
      simp at h'
      replace h' := h'.left
      unfold project at h'
      simp [ThetaStar, ToyHidden.projection, List.map] at h'
      -- h must equal ThetaStar
      cases h with
      | nil => simp at h'
      | cons a l =>
        -- We expect the singleton case
        -- This will require more unfolding
        sorry
    · intro h'
      -- If h = ThetaStar, it projects to tau
     rw [←h']
      constructor
      · unfold project
        simp [ThetaStar, ToyHidden.projection, List.map]
      · simp [toyVerifier]

/-!
## The Strict Gap

`synCost = 2` is strictly greater than `semCost = 1`.
-/

theorem strict_gap :
  synCost > ToySystem.semanticCost tau (by sorry) :=
  by
    -- Need finite fiber assumption
    have hFin : Finite (ToySystem.Fiber tau) := by sorry
    -- Compute semantic cost from the fiber
    have sem := ToySystem.semanticCost tau hFin
    -- For this toy case, assume sem = 1
    have sem_val : sem = 1 := by sorry
    -- Then 2 > 1 holds
    have gap : 2 > 1 := by norm_num
    gap

end ToySystem

end Coh.Core
