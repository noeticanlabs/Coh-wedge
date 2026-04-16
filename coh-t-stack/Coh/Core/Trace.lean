import Coh.Core.ReceiptChain
import Mathlib.Data.List.Basic

namespace Coh.Core

open Coh.Contract

abbrev Trace := List MicroReceipt

def nextStepIndex (start : Nat) : Trace → Nat
  | [] => start
  | _ :: rs => nextStepIndex (start + 1) rs

def finalChainDigest (start : ChainDigest) : Trace → ChainDigest
  | [] => start
  | r :: rs => finalChainDigest r.chainDigestNext rs

def finalStateHash (start : StateHash) : Trace → StateHash
  | [] => start
  | r :: rs => finalStateHash r.stateHashNext rs

inductive AcceptedTrace (cfg : ContractConfig) :
    Nat → StateHash → StateHash → ChainDigest → Trace → Prop
  | nil (startIndex : Nat) (startState : StateHash) (startDigest : ChainDigest) :
      AcceptedTrace cfg startIndex startState startState startDigest []
  | cons
      {startIndex : Nat}
      {startState midState endState : StateHash}
      {startDigest : ChainDigest}
      {r : MicroReceipt} {rs : Trace} :
      r.stepIndex = startIndex →
      AcceptedStep cfg startState midState startDigest r →
      AcceptedTrace cfg (startIndex + 1) midState endState r.chainDigestNext rs →
      AcceptedTrace cfg startIndex startState endState startDigest (r :: rs)

theorem acceptedTrace_head_tail
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState : StateHash}
    {startDigest : ChainDigest}
    {r : MicroReceipt} {rs : Trace}
    (h : AcceptedTrace cfg startIndex startState endState startDigest (r :: rs)) :
    ∃ midState,
      r.stepIndex = startIndex ∧
      AcceptedStep cfg startState midState startDigest r ∧
      AcceptedTrace cfg (startIndex + 1) midState endState r.chainDigestNext rs := by
  cases h with
  | cons hIdx hStep hTail =>
      exact ⟨_, hIdx, hStep, hTail⟩

theorem acceptedTrace_prefix
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState : StateHash}
    {startDigest : ChainDigest}
    {xs ys : Trace}
    (h : AcceptedTrace cfg startIndex startState endState startDigest (xs ++ ys)) :
    ∃ midState, AcceptedTrace cfg startIndex startState midState startDigest xs := by
  induction xs generalizing startIndex startState startDigest endState ys with
  | nil =>
      exact ⟨startState, AcceptedTrace.nil startIndex startState startDigest⟩
  | cons r xs ih =>
      have hCons : AcceptedTrace cfg startIndex startState endState startDigest (r :: (xs ++ ys)) := by
        simpa using h
      cases hCons with
      | cons hIdx hStep hTail =>
          obtain ⟨midState, hPrefixTail⟩ :=
            ih (startIndex := startIndex + 1)
              (startState := _)
              (startDigest := r.chainDigestNext)
              (endState := endState)
              (ys := ys)
              hTail
          exact ⟨midState, AcceptedTrace.cons hIdx hStep hPrefixTail⟩

theorem accepted_trace_closure
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState midState endState : StateHash}
    {startDigest : ChainDigest}
    {left right : Trace}
    (hLeft : AcceptedTrace cfg startIndex startState midState startDigest left)
    (hRight : AcceptedTrace cfg (nextStepIndex startIndex left) midState endState
      (finalChainDigest startDigest left) right) :
    AcceptedTrace cfg startIndex startState endState startDigest (left ++ right) := by
  induction hLeft generalizing endState right with
  | nil startIndex startState startDigest =>
      simpa [finalChainDigest, nextStepIndex] using hRight
  | @cons startIndex startState midState tailState startDigest r rs hIdx hStep hTail ih =>
      simp [finalChainDigest, nextStepIndex] at hRight
      exact AcceptedTrace.cons hIdx hStep (ih hRight)

/-!
## Trace Determinism

The following theorems establish that the final state hash of an
`AcceptedTrace` is uniquely determined by the initial state and the trace
sequence.  This rules out branching histories under the strict verifier.
-/

/-- Lemma: the end-state of any `AcceptedTrace` equals `finalStateHash`.
    Proof is by induction on the trace, using `rv_contract_correctness` to
    extract the forced `stateHashNext` at each accepted step. -/
theorem acceptedTrace_endState_eq_finalStateHash
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState : StateHash}
    {startDigest : ChainDigest}
    {t : Trace}
    (h : AcceptedTrace cfg startIndex startState endState startDigest t) :
    endState = finalStateHash startState t := by
  induction h with
  | nil startIndex startState startDigest =>
      simp [finalStateHash]
  | cons hIdx hStep hTail ih =>
      have hContract := (rv_contract_correctness _ _ _ _ _).mp hStep
      obtain ⟨_, _, _, _, _, _, _, hStateLink⟩ := hContract
      unfold stateHashLinkOK at hStateLink
      obtain ⟨_, hNext⟩ := hStateLink
      simp only [finalStateHash]
      rw [hNext]
      exact ih

/-- Determinism: two `AcceptedTrace` proofs for the same trace, start state,
    and start digest produce the same end state. -/
theorem acceptedTrace_endState_unique
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState₁ endState₂ : StateHash}
    {startDigest : ChainDigest}
    {t : Trace}
    (h₁ : AcceptedTrace cfg startIndex startState endState₁ startDigest t)
    (h₂ : AcceptedTrace cfg startIndex startState endState₂ startDigest t) :
    endState₁ = endState₂ :=
  (acceptedTrace_endState_eq_finalStateHash h₁).trans
    (acceptedTrace_endState_eq_finalStateHash h₂).symm

end Coh.Core


/-- The fundamental Chain Telescoping Theorem:
    Every AcceptedTrace satisfies the cumulative accounting law:
    v_post_last + totalSpend <= v_pre_first + totalDefect. --/
theorem chain_telescoping_theorem
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState : StateHash}
    {startDigest : ChainDigest}
    {t : Trace}
    (h : AcceptedTrace cfg startIndex startState endState startDigest t) :
    (t.getLast? >>= fun r => some r.metrics.vPost).getD (t.head? >>= fun r => some r.metrics.vPre).getD 0
    + totalSpend t <=
    (t.head? >>= fun r => some r.metrics.vPre).getD 0
    + totalDefect t := sorry
