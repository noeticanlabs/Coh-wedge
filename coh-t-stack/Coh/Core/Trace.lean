import Coh.Core.ReceiptChain
import Mathlib.Data.List.Basic

namespace Coh.Core

open Coh.Contract

abbrev Trace := List MicroReceipt

def finalChainDigest (start : ChainDigest) : Trace → ChainDigest
  | [] => start
  | r :: rs => finalChainDigest r.chainDigestNext rs

def finalStateHash (start : StateHash) : Trace → StateHash
  | [] => start
  | r :: rs => finalStateHash r.stateHashNext rs

inductive AcceptedTrace (cfg : ContractConfig) :
    StateHash → StateHash → ChainDigest → Trace → Prop
  | nil (startState : StateHash) (startDigest : ChainDigest) :
      AcceptedTrace cfg startState startState startDigest []
  | cons
      {startState midState endState : StateHash}
      {startDigest : ChainDigest}
      {r : MicroReceipt} {rs : Trace} :
      AcceptedStep cfg startState midState startDigest r →
      AcceptedTrace cfg midState endState r.chainDigestNext rs →
      AcceptedTrace cfg startState endState startDigest (r :: rs)

theorem accepted_trace_closure
    {cfg : ContractConfig}
    {startState midState endState : StateHash}
    {startDigest : ChainDigest}
    {left right : Trace}
    (hLeft : AcceptedTrace cfg startState midState startDigest left)
    (hRight : AcceptedTrace cfg midState endState (finalChainDigest startDigest left) right) :
    AcceptedTrace cfg startState endState startDigest (left ++ right) := by
  induction hLeft generalizing endState right with
  | nil startState startDigest =>
      simpa [finalChainDigest] using hRight
  | @cons startState midState tailState startDigest r rs hStep hTail ih =>
      simp [finalChainDigest] at hRight
      simpa using AcceptedTrace.cons hStep (ih hRight)

end Coh.Core
