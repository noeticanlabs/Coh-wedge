import Coh.Core.Trace
import Coh.Kernel.Receipt
import Mathlib.Data.Set.Basic
import Mathlib.Order.Directed

/-!
# Coh.Core.Semantic

This module introduces the semantic layer over certified traces.

It provides the paper-aligned formal objects for:
- hidden state and proposal
- projection from hidden traces to observable traces
- realizable fibers
- semantic cost over hidden realizations
- proof of semantic subadditivity

## Design Goals

- Keep exact alignment with the paper's theoretical definitions
- Minimize dependencies on other modules to prevent import cycles
- Provide a minimal coherent semantic kernel that can be extended later
- Mirror the key paper structures: X (observable), Hid (hidden), Pi (projection), W (hidden cost)
-/

namespace Coh.Core

/-!
## Semantic System

A semantic system consists of:
- a hidden state space (Hid)
- an observable state space (X)
- a projection map (Pi : Hid → X)
- a hidden cost functional (W : Hid → ℝ≥0)
- a proposal/proposal space (Hist) for generating hidden traces
- a potential on observable states (V : X → ℝ≥0)
- A verifier type (RV) that operates on verifiable traces

This structure mirrors the paper's Coh system definition.
-/

/-- A semantic system with hidden and observable layers. -/
structure SemanticSystem (H X : Type) where
  hid_space : H
  obs_space : X
  projection : H → X
  hid_cost : H → NNReal
  hist_space : Type
  proposal : H → hist_space → List (H × H) -- hidden trace as list of (src, tgt) pairs
  valuation : X → NNReal
  verifier : (List (H × H)) → Prop -- verifies hidden traces

namespace SemanticSystem

variable {H X : Type} (S : SemanticSystem H X)

/-!
## Hidden Traces

A hidden trace is a list of pairs (src, tgt) representing pairs of hidden states.

This corresponds to the paper's Θ hidden trajectory.
-/

/-- Hidden trace as a sequence of hidden states. -/
def HiddenTrace := List H

/-!
## Observable Projection

Project a hidden trace down to observable states.

This defines the function Π(Θ) in the paper.
-/

/-- Project a hidden trace to observable states by applying projection pointwise. -/
def project (h : HiddenTrace) : List X :=
  h.map S.projection

/-!
## Realizable Fiber

Given an observable trace τ, the realizable fiber is the set of hidden traces that project to τ
and are accepted by the verifier.

This defines Rset(τ) in the paper.
-/

/-- The fiber of hidden traces realizing an observable trace. -/
def Fiber (obs : List X) : Set HiddenTrace :=
  { h : HiddenTrace | S.project h = obs ∧ S.verifier h }

/-!
## Semantic Cost

Define semantic cost over the fiber as the maximum hidden cost (per the paper).

We assume finite realizable fibers for now; this can be relaxed if needed.
-/

/--
Semantic cost of an observable trace. Defined as the maximum hidden cost over realizations.
Requires a finite fiber assumption for well-definedness.
-/
def semanticCost (obs : List X) (hFin : Finite (S.Fiber obs)) : NNReal :=
  (S.Fiber obs).toFinset.sup id S.hid_cost

/-!
## Properties

We prove compatibility and subadditivity lemmas.
-/

/-- Project of concatenated hidden traces equals concatenation of projections. -/
theorem project_concat (h1 h2 : HiddenTrace) :
  S.project (h1 ++ h2) = S.project h1 ++ S.project h2 :=
  by simp [project, List.map_append]

/-!
## Semantic Subadditivity

This corresponds to the paper's main result:
`s_sem(τ2 ∘ τ1) <= s_sem(τ1) + s_sem(τ2)`
-/

/--
Semantic subadditivity: cost of composed traces is bounded by sum of costs.
[HYPOTHESIS - depends on finiteness of fibers]
-/
theorem semantic_subadditive (obs1 obs2 : List X)
    (hFin1 : Finite (S.Fiber obs1))
    (hFin2 : Finite (S.Fiber obs2))
    (hFin12 : Finite (S.Fiber (obs1 ++ obs2))) :
  S.semanticCost (obs1 ++ obs2) hFin12 ≤
  S.semanticCost obs1 hFin1 + S.semanticCost obs2 hFin2 :=
  by
  sorry -- TODO: prove using finite supremum over restricted fibers

end SemanticSystem

/-!
## Alternative Construction: From Certified Traces

We can also construct the semantic layer directly from the existing CertifiedTrace.

This provides a bridge to the existing implementation.
-/

namespace SemanticFromTrace

/-- Wrap an accepted trace as an observable trace in our system. -/
def obsof {cfg : ContractConfig}
    {c : ChainDigest}
    {st idx : Nat}
    {sh : StateHash}
    {t : Trace}
    (h : AcceptedTrace cfg idx st sh c t) : List X :=
  t

end SemanticFromTrace

end Coh.Core
