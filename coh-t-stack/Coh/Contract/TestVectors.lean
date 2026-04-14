import Coh.Contract.Slab
import Coh.Contract.Canon

namespace Coh.Contract

open Coh.Core

def sampleConfig : ContractConfig :=
  { microSchema := "coh.receipt.micro.v1"
    microVersion := "1.0.0"
    slabSchema := "coh.receipt.slab.v1"
    slabVersion := "1.0.0"
    canonProfileHash := "profile.v1" }

def genesisDigest : ChainDigest :=
  ⟨"GENESIS"⟩

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

def badSchemaMicro : MicroReceipt :=
  { sampleMicro with schemaId := "bad.schema" }

def emptyObjectIdMicro : MicroReceipt :=
  { sampleMicro with objectId := "" }

def badProfileMicro : MicroReceipt :=
  { sampleMicro with canonProfileHash := "wrong-profile" }

def badNumericMicro : MicroReceipt :=
  { sampleMicro with
      metrics :=
        { vPre := u128Max + 1
          vPost := 0
          spend := 0
          defect := 0 } }

def badPolicyMicro : MicroReceipt :=
  { sampleMicro with
      metrics :=
        { vPre := 0
          vPost := 5
          spend := 1
          defect := 0 } }

def badDigestMicro : MicroReceipt :=
  { sampleMicro with chainDigestNext := ⟨"wrong-digest"⟩ }

def badStateLinkMicro : MicroReceipt :=
  { sampleMicro with stateHashPrev := ⟨"wrong-prev"⟩ }

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

def emptySlab : SlabReceipt :=
  { sampleSlab with microCount := 0 }

def badRangeSlab : SlabReceipt :=
  { sampleSlab with rangeStart := 2, rangeEnd := 0 }

def badCountSlab : SlabReceipt :=
  { sampleSlab with microCount := 2 }

def overflowSlab : SlabReceipt :=
  { sampleSlab with
      summary :=
        { totalSpend := 1
          totalDefect := 0
          vPreFirst := 0
          vPostLast := u128Max } }

def badSummarySlab : SlabReceipt :=
  { sampleSlab with
      summary :=
        { totalSpend := 10
          totalDefect := 0
          vPreFirst := 1
          vPostLast := 5 } }

def badMerkleSlab : SlabReceipt :=
  { sampleSlab with merkleWitnessValid := false }

example : rv sampleConfig samplePrevState sampleNextState genesisDigest sampleMicro = true := by
  native_decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest sampleMicro = none := by
  apply verifyMicroRejectCode_none_of_contract
  native_decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badSchemaMicro = false := by
  apply rv_reject_of_bad_schema
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badSchemaMicro =
    some RejectCode.rejectSchema := by
  apply verifyMicroRejectCode_of_bad_schema
  decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest emptyObjectIdMicro = false := by
  apply rv_reject_of_empty_object_id
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest emptyObjectIdMicro =
    some RejectCode.rejectSchema := by
  apply verifyMicroRejectCode_of_empty_object_id
  all_goals decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badProfileMicro = false := by
  apply rv_reject_of_bad_canon_profile
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badProfileMicro =
    some RejectCode.rejectCanonProfile := by
  apply verifyMicroRejectCode_of_bad_canon_profile
  all_goals decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badNumericMicro = false := by
  apply rv_reject_of_numeric_invalid
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badNumericMicro =
    some RejectCode.rejectNumericParse := by
  apply verifyMicroRejectCode_of_numeric_parse
  all_goals decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badPolicyMicro = false := by
  apply rv_reject_of_policy_violation
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badPolicyMicro =
    some RejectCode.rejectPolicyViolation := by
  apply verifyMicroRejectCode_of_policy_violation
  all_goals decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badDigestMicro = false := by
  apply rv_reject_of_bad_chain_digest
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badDigestMicro =
    some RejectCode.rejectChainDigest := by
  apply verifyMicroRejectCode_of_bad_chain_digest
  all_goals decide

example : rv sampleConfig samplePrevState sampleNextState genesisDigest badStateLinkMicro = false := by
  apply rv_reject_of_bad_state_link
  decide

example : verifyMicroRejectCode sampleConfig samplePrevState sampleNextState genesisDigest badStateLinkMicro =
    some RejectCode.rejectStateHashLink := by
  apply verifyMicroRejectCode_of_bad_state_link
  all_goals decide

def canonicalPayloadMicro : MicroReceipt :=
  { sampleMicro with canonicalPayload := canonicalMicroJson sampleMicro }

example : PayloadMatchesCanonicalJson canonicalPayloadMicro := by
  unfold PayloadMatchesCanonicalJson canonicalPayloadMicro
  rfl

example : verifySlabEnvelope sampleConfig sampleSlab = true := by
  apply verify_slab_envelope_accept_of_valid_summary
  all_goals native_decide

example : verifySlabEnvelopeRejectCode sampleConfig sampleSlab = none := by
  apply verifySlabEnvelopeRejectCode_none_of_valid_summary
  all_goals native_decide

example : verifySlab sampleConfig sampleSlab = true := by
  native_decide

example : verifySlabRejectCode sampleConfig sampleSlab = none := by
  apply verifySlabRejectCode_none_of_valid_merkle_summary
  all_goals native_decide

example : verifySlabEnvelope sampleConfig emptySlab = false := by
  apply verify_slab_envelope_reject_of_empty
  decide

example : verifySlabEnvelopeRejectCode sampleConfig emptySlab = some RejectCode.rejectSlabSummary := by
  apply verifySlabEnvelopeRejectCode_of_empty
  all_goals decide

example : verifySlabEnvelope sampleConfig badRangeSlab = false := by
  apply verify_slab_envelope_reject_of_invalid_range
  decide

example : verifySlabEnvelopeRejectCode sampleConfig badRangeSlab = some RejectCode.rejectSlabSummary := by
  apply verifySlabEnvelopeRejectCode_of_invalid_range
  all_goals decide

example : verifySlabEnvelope sampleConfig badCountSlab = false := by
  apply verify_slab_envelope_reject_of_bad_count
  decide

example : verifySlabEnvelopeRejectCode sampleConfig badCountSlab = some RejectCode.rejectSlabSummary := by
  apply verifySlabEnvelopeRejectCode_of_bad_count
  all_goals decide

example : verifySlabEnvelope sampleConfig overflowSlab = false := by
  apply verify_slab_envelope_reject_of_overflow
  decide

example : verifySlabEnvelopeRejectCode sampleConfig overflowSlab = some RejectCode.rejectOverflow := by
  apply verifySlabEnvelopeRejectCode_of_overflow
  all_goals decide

example : verifySlab sampleConfig badSummarySlab = false := by
  apply verify_slab_reject_of_wrong_summary
  decide

example : verifySlabEnvelopeRejectCode sampleConfig badSummarySlab = some RejectCode.rejectPolicyViolation := by
  apply verifySlabEnvelopeRejectCode_of_policy_violation
  all_goals decide

example : verifySlab sampleConfig badMerkleSlab = false := by
  apply verify_slab_reject_of_bad_merkle
  decide

example : verifySlabRejectCode sampleConfig badMerkleSlab = some RejectCode.rejectSlabMerkle := by
  apply verifySlabRejectCode_of_bad_merkle
  all_goals native_decide

end Coh.Contract
