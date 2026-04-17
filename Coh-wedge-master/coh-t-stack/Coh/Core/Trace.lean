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

/-!
## Aggregate Accounting Functions (T3)

These define the total spend and total defect across a trace.
They are used by the Chain Telescoping Theorem to state cumulative bounds.
-/

/-- Total spend accumulated across all steps in a trace. -/
def totalSpend (t : Trace) : Nat :=
  t.foldl (fun acc r => acc + r.metrics.spend) 0

/-- Total defect bound accumulated across all steps in a trace. -/
def totalDefect (t : Trace) : Nat :=
  t.foldl (fun acc r => acc + r.metrics.defect) 0

/-!
## Metric Continuity (needed for telescoping)

Telescoping the aggregate inequality v_post_last + totalSpend ≤ v_pre_first + totalDefect
requires that the post-value of each step equals the pre-value of the next step:
`∀ i, step[i].vPost = step[i+1].vPre`. Without this continuity, the telescoping
cancellation does not apply and the theorem is false.
-/

/-- A trace is metric-continuous when every step's post-value equals the next step's pre-value.
    This is the necessary condition for the telescoping inequality to hold across the trace. -/
def MetricsContinuous : Trace → Prop
  | [] => True
  | [r] => True
  | r1 :: r2 :: rs =>
    r1.metrics.vPost = r2.metrics.vPre ∧ MetricsContinuous (r2 :: rs)

instance : Decidable (MetricsContinuous []) := by unfold MetricsContinuous; infer_instance
instance : Decidable (MetricsContinuous [r]) := by unfold MetricsContinuous; infer_instance
instance (r1 r2 : MicroReceipt) (rs : Trace) : Decidable (MetricsContinuous (r1 :: r2 :: rs)) :=
  by unfold MetricsContinuous; infer_instance

/-! Helper lemmas for totals and step-level properties -/

@[simp] lemma totalSpend_nil : totalSpend [] = 0 := rfl
@[simp] lemma totalDefect_nil : totalDefect [] = 0 := rfl

lemma totalSpend_cons (r : MicroReceipt) (rs : Trace) :
  totalSpend (r :: rs) = r.metrics.spend + totalSpend rs :=
  by simp [totalSpend, List.foldl_cons, Nat.add_comm]

lemma totalDefect_cons (r : MicroReceipt) (rs : Trace) :
  totalDefect (r :: rs) = r.metrics.defect + totalDefect rs :=
  by simp [totalDefect, List.foldl_cons, Nat.add_comm]

/-- Continuity Ergonomics: The head's vPre of a continuous cons is stable. -/
lemma head_pre_of_continuous_cons (r : MicroReceipt) (rs : Trace) (h : MetricsContinuous (r :: rs)) :
  ( (r :: rs).head? >>= fun h => some h.metrics.vPre ).getD 0 = r.metrics.vPre :=
  by simp [List.head?]

/-- Continuity Ergonomics: The last post of a cons list is the last post of the tail, or the head's if tail is empty. -/
lemma last_post_of_cons (r : MicroReceipt) (rs : Trace) :
  ( (r :: rs).getLast? >>= fun h => some h.metrics.vPost ).getD 0 =
    ( rs.getLast? >>= fun h => some h.metrics.vPost ).getD r.metrics.vPost :=
  by cases rs <;> simp [List.getLast?]

/-- If a step is accepted, it obeys the per-step policy law (vPost + spend ≤ vPre + defect). -/
theorem accepted_step_policyLawful
    {cfg : ContractConfig}
    {prevState nextState : StateHash}
    {prevChainDigest : ChainDigest}
    (r : MicroReceipt)
    (hStep : AcceptedStep cfg prevState prevChainDigest r) :
    r.metrics.vPost + r.metrics.spend ≤ r.metrics.vPre + r.metrics.defect :=
  have hContract := (rv_contract_correctness cfg prevState nextState prevChainDigest r).mp hStep
  obtain ⟨_,_,_,_,hPolicy,_,_,_⟩ := hContract
  hPolicy

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


/-/ The fundamental Chain Telescoping Theorem statement (proof now complete): -/

/-- The fundamental Chain Telescoping Theorem (with continuity hypothesis):
    Every `AcceptedTrace` that is metric-continuous satisfies the cumulative law:
    v_post_last + totalSpend ≤ v_pre_first + totalDefect.

    Continuity (post of step i = pre of step i+1) is required for telescoping
    the per-step inequalities into a single global bound. Without this
    hypothesis, the cancellation fails. -/
theorem chain_telescoping_theorem
    {cfg : ContractConfig}
    {startIndex : Nat}
    {startState endState : StateHash}
    {startDigest : ChainDigest}
    {t : Coh.Core.Trace}
    (h : Coh.Core.AcceptedTrace cfg startIndex startState endState startDigest t)
    (hCont : Coh.Core.MetricsContinuous t) :
    (t.getLast? >>= fun r => some r.metrics.vPost).getD (t.head? >>= fun r => some r.metrics.vPre).getD 0
    + Coh.Core.totalSpend t <=
    (t.head? >>= fun r => some r.metrics.vPre).getD 0
    + Coh.Core.totalDefect t :=
  by
  induction h with
  | nil _ _ _ =>
    simp [totalSpend, totalDefect]
  | @cons startIndex startState midState endState startDigest r rs hIdx hStep hTail ih =>
    -- Use ergonomics lemmas
    rw [totalSpend_cons, totalDefect_cons, last_post_of_cons]
    rw [head_pre_of_continuous_cons r rs hCont]
    -- Step policy: r.vPost + r.spend <= r.vPre + r.defect
    have hStepPolicy := accepted_step_policyLawful r hStep
    -- Handle continuity: if rs is nonempty, r.vPost = rs.head.vPre
    cases rs with
    | nil =>
      simp [totalSpend, totalDefect] at *
      exact hStepPolicy
    | cons r2 rs' =>
      -- Continuity: r.vPost = r2.vPre
      have hCont' : MetricsContinuous (r :: r2 :: rs') := hCont
      unfold MetricsContinuous at hCont'
      obtain ⟨hEq, hContTail⟩ := hCont'
      -- Apply IH to the tail
      have ih' := ih hContTail
      -- Synchronize head of tail with r.vPost
      simp [List.head?] at ih'
      rw [hEq] at hStepPolicy
      -- Now combines ih' and hStepPolicy using Nat.add_le_add
      have hCombined := Nat.add_le_add hStepPolicy ih'
      -- Rearrange for the goal
      simp [Nat.add_assoc] at *
      linarith
