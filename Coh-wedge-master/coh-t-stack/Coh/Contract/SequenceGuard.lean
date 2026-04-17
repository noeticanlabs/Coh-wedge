import Mathlib.Data.ByteArray
import Coh.Contract.MicroV3
import Coh.Contract.PolicyGovernance

/-!
# Coh.Contract.SequenceGuard

Sequence/Temporal Guard: rolling checks to prevent temporal attacks.

This module extends the contract layer with:
- Rolling accumulator: track metrics over a window
- State drift check: detect rapid state changes
- Cumulative spend check: prevent unbounded spend
- Custom invariants: user-defined rolling constraints

This addresses Gap 3: "Time / Ordering Attacks"
- Single-step verification doesn't catch sequences
- Sequences of valid steps can be catastrophic together
- Rolling constraints detect drift over time

Key insight: We verify not just "is this step valid?" but
"is this step valid given the recent history?"
-/

namespace Coh.Contract

open Coh.Contract

/-- Sequence guard configuration -/
structure SequenceGuard where
  /-- Maximum cumulative spend in rolling window -/
  maxCumulativeSpend : Nat
  /-- Window size in steps -/
  windowSize : Nat
  /-- Maximum absolute state drift (change magnitude) in window -/
  maxStateDrift : Nat
  /-- Maximum value decrease rate per step -/
  maxValueDecreasePerStep : Nat
  /-- Require monotonicity (value never decreases below threshold) -/
  requireMonotonicity : Bool
  deriving Repr, DecidableEq

/-- Default permissive sequence guard -/
def defaultSequenceGuard : SequenceGuard :=
  { maxCumulativeSpend := u128Max.toNat
    windowSize := 100
    maxStateDrift := u128Max.toNat
    maxValueDecreasePerStep := u128Max.toNat
    requireMonotonicity := false }

/-- Strict sequence guard -/
def strictSequenceGuard : SequenceGuard :=
  { maxCumulativeSpend := 10000
    windowSize := 10
    maxStateDrift := 5000
    maxValueDecreasePerStep := 1000
    requireMonotonicity := true }

/-- Accumulated metrics over a window -/
structure WindowMetrics where
  stepCount : Nat
  totalSpend : Nat
  totalValueDecrease : Nat
  minValue : Nat
  maxValue : Nat
  deriving Repr

/-- Empty window metrics -/
def emptyWindowMetrics : WindowMetrics :=
  { stepCount := 0
    totalSpend := 0
    totalValueDecrease := 0
    minValue := u128Max.toNat
    maxValue := 0 }

/-- Add a receipt to window metrics -/
def WindowMetrics.add (m : WindowMetrics) (r : MicroReceiptV3) : WindowMetrics :=
  let spend := r.metrics.spend.toNat
  let valueDiff := if r.metrics.vPre > r.metrics.vPost
                 then r.metrics.vPre.toNat - r.metrics.vPost.toNat
                 else 0
  { stepCount := m.stepCount + 1
    totalSpend := m.totalSpend + spend
    totalValueDecrease := m.totalValueDecrease + valueDiff
    minValue := min m.minValue r.metrics.vPost.toNat
    maxValue := max m.maxValue r.metrics.vPre.toNat }

/-- Compute window from recent receipts (takes last N) -/
def computeWindowMetrics (guard : SequenceGuard)
    (receipts : List MicroReceiptV3) : WindowMetrics :=
  let window := receipts.reverse.take guard.windowSize
  window.foldl WindowMetrics.add emptyWindowMetrics

/-- Check cumulative spend within limit -/
def cumulativeSpendValid (guard : SequenceGuard)
    (window : WindowMetrics) : Bool :=
  window.totalSpend ≤ guard.maxCumulativeSpend

/-- Check state drift within limit -/
def stateDriftValid (guard : SequenceGuard)
    (window : WindowMetrics) : Bool :=
  window.totalValueDecrease ≤ guard.maxStateDrift

/-- Check monotonicity requirement -/
def monotonicityValid (guard : SequenceGuard)
    (window : WindowMetrics) : Bool :=
  if guard.requireMonotonicity then
    window.minValue ≥ 500  -- threshold for "too low"
  else true

/-- Combined sequence guard validity -/
def sequenceGuardValid (guard : SequenceGuard)
    (window : WindowMetrics) : Bool :=
  cumulativeSpendValid guard window ∧
  stateDriftValid guard window ∧
  monotonicityValid guard window

/-- Sequence guard reject code -/
inductive SequenceRejectCode where
  | cumSpendExceeded : SequenceRejectCode
  | stateDriftExceeded : SequenceRejectCode
  | monotonicityViolation : SequenceRejectCode
  | windowUnderflow : SequenceRejectCode

/-- Verify sequence guard for given history -/
def verifySequenceGuard
    (guard : SequenceGuard)
    (receiptHistory : List MicroReceiptV3)
    (currentReceipt : MicroReceiptV3)
    : Option SequenceRejectCode :=
  /-- Need enough history for window -/
  if receiptHistory.length < guard.windowSize then
    some SequenceRejectCode.windowUnderflow
  else
    let window := computeWindowMetrics guard receiptHistory
    let currentWindow := window.add currentReceipt
    if ¬ cumulativeSpendValid guard currentWindow then
      some SequenceRejectCode.cumSpendExceeded
    else if ¬ stateDriftValid guard currentWindow then
      some SequenceRejectCode.stateDriftExceeded
    else if ¬ monotonicityValid guard currentWindow then
      some SequenceRejectCode.monotonicityViolation
    else none

/-- Custom invariant type -/
structure CustomInvariant where
  name : String
  /-- Check function: given window, return whether invariant holds -/
  check : WindowMetrics → Bool

/-- Example: total spend not exceed double current spend -/
def doubleSpendInvariant (currentSpend : Nat) : CustomInvariant :=
  { name := "doubleSpend"
    check := fun w => w.totalSpend ≤ 2 * currentSpend }

/-- Example: value doesn't drop below 20% of max -/
def twentyPercentInvariant : CustomInvariant :=
  { name := "twentyPercent"
    check := fun w =>
      if w.maxValue > 0 then
        w.minValue ≥ w.maxValue / 5
      else true }

/-- Combine multiple invariants -/
def checkAllInvariants (invariants : List CustomInvariant)
    (window : WindowMetrics) : Bool :=
  invariants.all (fun inv => inv.check window)

/-- Extended sequence guard with custom invariants -/
structure SequenceGuardExt where
  base : SequenceGuard
  invariants : List CustomInvariant
  deriving Repr

/-- Verify extended sequence guard -/
def verifySequenceGuardExt
    (guard : SequenceGuardExt)
    (receiptHistory : List MicroReceiptV3)
    (currentReceipt : MicroReceiptV3)
    : Option SequenceRejectCode :=
  match verifySequenceGuard guard.base receiptHistory currentReceipt with
  | some code => some code
  none =>
    let window := computeWindowMetrics guard.base receiptHistory
    let currentWindow := window.add currentReceipt
    if ¬ checkAllInvariants guard.invariants currentWindow then
      some SequenceRejectCode.stateDriftExceeded  -- reused code
    else none

/-- Sequence context: accumulated state for verification -/
structure SequenceContext where
  /-- Recent receipt history (full chain) -/
  receiptHistory : List MicroReceiptV3
  /-- Current sequence guard config -/
  guard : SequenceGuard
  deriving Repr

/-- Empty sequence context -/
def emptySequenceContext : SequenceContext :=
  { receiptHistory := []
    guard := defaultSequenceGuard }

/-- Add receipt to context -/
def SequenceContext.add (ctx : SequenceContext)
    (r : MicroReceiptV3) : SequenceContext :=
  { ctx with
    receiptHistory := ctx.receiptHistory.append [r] }

/-- Verify receipt with sequence context -/
def verifyWithSequenceGuard
    (ctx : SequenceContext)
    (prevState nextState prevChainDigest : Digest)
    (r : MicroReceiptV3) : Decision RejectCode :=
  /-- First verify receipt validity -/
  let baseResult := verifyMicroV3RejectCode defaultBoundaryConfig prevState nextState prevChainDigest r
  match baseResult with
  | some code => Decision.reject code
  none =>
    /-- Then verify sequence guard -/
    match verifySequenceGuard ctx.guard ctx.receiptHistory r with
    | some SequenceRejectCode.cumSpendExceeded =>
      Decision.reject RejectCode.rejectPolicyViolation
    | some SequenceRejectCode.stateDriftExceeded =>
      Decision.reject RejectCode.rejectPolicyViolation
    | some SequenceRejectCode.monotonicityViolation =>
      Decision.reject RejectCode.rejectPolicyViolation
    | some SequenceRejectCode.windowUnderflow =>
      /- Allow if not enough history, but log -/
      Decision.accept
    | none => Decision.accept

/-- Example: empty window is valid -/
example : cumulativeSpendValid defaultSequenceGuard emptyWindowMetrics := by
  unfold cumulativeSpendValid defaultSequenceGuard emptyWindowMetrics
  simp [u128Max.toNat]

/-- Example: adding a receipt increases step count -/
example : (emptyWindowMetrics.add sampleMicroV3).stepCount = 1 := by
  unfold WindowMetrics.add sampleMicroV3
  simp [emptyWindowMetrics, sampleMicroV3.metrics.spend.toNat]

/-- Example: sequence underflow when history too short -/
example : verifySequenceGuard strictSequenceGuard [] sampleMicroV3
  = some SequenceRejectCode.windowUnderflow := by
  unfold verifySequenceGuard
  simp [strictSequenceGuard, List.length]

end Coh.Contract
