import Coh.Contract.Profile
import Coh.Contract.Schema
import Coh.Contract.RejectCode
import Coh.Core.Hash

namespace Coh.Contract

open Coh.Core

structure Metrics where
  vPre : Nat
  vPost : Nat
  spend : Nat
  defect : Nat
  deriving Repr, DecidableEq

structure ContractConfig where
  microSchema : SchemaId
  microVersion : String
  slabSchema : SchemaId
  slabVersion : String
  canonProfileHash : CanonProfileHash
  deriving Repr, DecidableEq

structure MicroReceipt where
  schemaId : SchemaId
  version : String
  objectId : String
  canonProfileHash : CanonProfileHash
  policyHash : PolicyHash
  stepIndex : Nat
  stateHashPrev : StateHash
  stateHashNext : StateHash
  chainDigestPrev : ChainDigest
  chainDigestNext : ChainDigest
  canonicalPayload : String
  metrics : Metrics
  deriving Repr, DecidableEq

namespace MicroReceipt

def ValidSchema (cfg : ContractConfig) (r : MicroReceipt) : Prop :=
  r.schemaId = cfg.microSchema ∧ r.version = cfg.microVersion

end MicroReceipt

def CanonProfilePinned (cfg : ContractConfig) (r : MicroReceipt) : Prop :=
  r.canonProfileHash = cfg.canonProfileHash

def NumericValid (_r : MicroReceipt) : Prop :=
  True

def policyLawful (r : MicroReceipt) : Prop :=
  r.metrics.vPost + r.metrics.spend ≤ r.metrics.vPre + r.metrics.defect

def stateHashLinkOK (prevState nextState : StateHash) (r : MicroReceipt) : Prop :=
  r.stateHashPrev = prevState ∧ r.stateHashNext = nextState

def chainDigestMatches (r : MicroReceipt) : Prop :=
  r.chainDigestNext = digestUpdate r.chainDigestPrev r.canonicalPayload

def microContractPred
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Prop :=
  MicroReceipt.ValidSchema cfg r ∧
    CanonProfilePinned cfg r ∧
    NumericValid r ∧
    policyLawful r ∧
    r.chainDigestPrev = prevChainDigest ∧
    chainDigestMatches r ∧
    stateHashLinkOK prevState nextState r

def rv
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Bool :=
  decide (microContractPred cfg prevState nextState prevChainDigest r)

theorem rv_contract_correctness
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) :
    rv cfg prevState nextState prevChainDigest r = true ↔
      MicroReceipt.ValidSchema cfg r ∧
        CanonProfilePinned cfg r ∧
        NumericValid r ∧
        policyLawful r ∧
        r.chainDigestPrev = prevChainDigest ∧
        chainDigestMatches r ∧
        stateHashLinkOK prevState nextState r := by
  unfold rv microContractPred
  simp

end Coh.Contract
