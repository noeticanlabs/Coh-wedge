import Coh.Contract.Slab

namespace Coh.Contract

open Coh.Core

def sampleConfig : ContractConfig :=
  { microSchema := "coh.receipt.micro.v1"
    microVersion := "1.0.0"
    slabSchema := "coh.receipt.slab.v1"
    slabVersion := "1.0.0"
    canonProfileHash := "profile.v1" }

def genesisDigest : ChainDigest :=
  ChainDigest.genesis "GENESIS"

def samplePrevState : StateHash :=
  ⟨"state-0"⟩

def sampleNextState : StateHash :=
  ⟨"state-1"⟩

def sampleMetrics : Metrics :=
  { vPre := 10, vPost := 7, spend := 3, defect := 0 }

def sampleMicro : MicroReceipt :=
  { schemaId := "coh.receipt.micro.v1"
    version := "1.0.0"
    objectId := "object-0"
    canonProfileHash := "profile.v1"
    policyHash := "policy.v1"
    stepIndex := 0
    stateHashPrev := samplePrevState
    stateHashNext := sampleNextState
    chainDigestPrev := genesisDigest
    chainDigestNext := digestUpdate genesisDigest "payload-0"
    canonicalPayload := "payload-0"
    metrics := sampleMetrics }

def sampleSlab : SlabReceipt :=
  { schemaId := "coh.receipt.slab.v1"
    version := "1.0.0"
    objectId := "object-0"
    canonProfileHash := "profile.v1"
    policyHash := "policy.v1"
    rangeStart := 0
    rangeEnd := 0
    microCount := 1
    chainDigestPrev := genesisDigest
    chainDigestNext := digestUpdate genesisDigest "payload-0"
    stateHashFirst := samplePrevState
    stateHashLast := sampleNextState
    merkleRoot := "merkle-root"
    merkleWitnessValid := true
    summary :=
      { totalSpend := 3
        totalDefect := 0
        vPreFirst := 10
        vPostLast := 7 } }

example : rv sampleConfig samplePrevState sampleNextState genesisDigest sampleMicro = true := by
  native_decide

example : verifySlab sampleConfig sampleSlab = true := by
  native_decide

end Coh.Contract
