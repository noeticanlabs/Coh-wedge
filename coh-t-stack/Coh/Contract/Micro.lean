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
  authority : Nat
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

instance instDecidableValidSchema (cfg : ContractConfig) (r : MicroReceipt) :
    Decidable (ValidSchema cfg r) := by
  unfold ValidSchema
  infer_instance

end MicroReceipt

def CanonProfilePinned (cfg : ContractConfig) (r : MicroReceipt) : Prop :=
  r.canonProfileHash = cfg.canonProfileHash

instance instDecidableCanonProfilePinned (cfg : ContractConfig) (r : MicroReceipt) :
    Decidable (CanonProfilePinned cfg r) := by
  unfold CanonProfilePinned
  infer_instance

/-- Rust numeric domain bound induced by `u128` parsing in the verifier. -/
def u128Max : Nat :=
  340282366920938463463374607431768211455

/-- Object identifiers must be nonempty, matching the Rust sanity check. -/
def ObjectIdValid (r : MicroReceipt) : Prop :=
  r.objectId ≠ ""

instance instDecidableObjectIdValid (r : MicroReceipt) : Decidable (ObjectIdValid r) := by
  unfold ObjectIdValid
  infer_instance

/-- Parsed metric values must lie within the `u128` range accepted by Rust. -/
def MetricsParseValid (r : MicroReceipt) : Prop :=
  r.metrics.vPre ≤ u128Max ∧
    r.metrics.vPost ≤ u128Max ∧
    r.metrics.spend ≤ u128Max ∧
    r.metrics.defect ≤ u128Max ∧
    r.metrics.authority ≤ u128Max

instance instDecidableMetricsParseValid (r : MicroReceipt) : Decidable (MetricsParseValid r) := by
  unfold MetricsParseValid
  infer_instance

/-- Checked additions used by the Rust verifier must stay within the `u128` domain. -/
def MetricsNoOverflow (r : MicroReceipt) : Prop :=
  r.metrics.vPost + r.metrics.spend ≤ u128Max ∧
    r.metrics.vPre + r.metrics.defect + r.metrics.authority ≤ u128Max

instance instDecidableMetricsNoOverflow (r : MicroReceipt) : Decidable (MetricsNoOverflow r) := by
  unfold MetricsNoOverflow
  infer_instance

/-- Numeric validity combines parse-range and checked-addition requirements. -/
def NumericValid (r : MicroReceipt) : Prop :=
  MetricsParseValid r ∧ MetricsNoOverflow r

instance instDecidableNumericValid (r : MicroReceipt) : Decidable (NumericValid r) :=
  by
    unfold NumericValid
    infer_instance

def policyLawful (r : MicroReceipt) : Prop :=
  r.metrics.vPost + r.metrics.spend ≤ r.metrics.vPre + r.metrics.defect + r.metrics.authority

instance instDecidablePolicyLawful (r : MicroReceipt) : Decidable (policyLawful r) := by
  unfold policyLawful
  infer_instance

/-- Domain constraints matching Rust: spend <= vPre and no vacuous zeros. -/
def domainLawful (r : MicroReceipt) : Prop :=
  r.metrics.spend ≤ r.metrics.vPre ∧
  ¬ (r.metrics.vPre = 0 ∧ r.metrics.vPost = 0 ∧ r.metrics.spend = 0 ∧ r.metrics.defect = 0 ∧ r.metrics.authority = 0)

instance instDecidableDomainLawful (r : MicroReceipt) : Decidable (domainLawful r) := by
  unfold domainLawful
  infer_instance

def stateHashLinkOK (prevState nextState : StateHash) (r : MicroReceipt) : Prop :=
  r.stateHashPrev = prevState ∧ r.stateHashNext = nextState

instance instDecidableStateHashLinkOK
    (prevState nextState : StateHash) (r : MicroReceipt) :
    Decidable (stateHashLinkOK prevState nextState r) := by
  unfold stateHashLinkOK
  infer_instance

def chainDigestMatches (r : MicroReceipt) : Prop :=
  r.chainDigestNext = digestUpdate r.chainDigestPrev r.canonicalPayload

instance instDecidableChainDigestMatches (r : MicroReceipt) :
    Decidable (chainDigestMatches r) := by
  unfold chainDigestMatches
  infer_instance

def microContractPred
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Prop :=
  MicroReceipt.ValidSchema cfg r ∧
    CanonProfilePinned cfg r ∧
    ObjectIdValid r ∧
    NumericValid r ∧
    policyLawful r ∧
    domainLawful r ∧
    r.chainDigestPrev = prevChainDigest ∧
    chainDigestMatches r ∧
    stateHashLinkOK prevState nextState r

instance instDecidableMicroContractPred
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) :
    Decidable (microContractPred cfg prevState nextState prevChainDigest r) := by
  unfold microContractPred
  infer_instance

def rv
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Bool :=
  decide (microContractPred cfg prevState nextState prevChainDigest r)

def numericRejectCode (r : MicroReceipt) : Option RejectCode :=
  if ¬ MetricsParseValid r then some RejectCode.rejectNumericParse
  else if ¬ MetricsNoOverflow r then some RejectCode.rejectOverflow
  else none

def verifyMicroRejectCode
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Option RejectCode :=
  if ¬ MicroReceipt.ValidSchema cfg r then some RejectCode.rejectSchema
  else if ¬ ObjectIdValid r then some RejectCode.rejectSchema
  else if ¬ CanonProfilePinned cfg r then some RejectCode.rejectCanonProfile
  else match numericRejectCode r with
    | some code => some code
    | none =>
        if ¬ policyLawful r then some RejectCode.rejectPolicyViolation
        else if r.metrics.spend > r.metrics.vPre then some RejectCode.spendExceedsBalance
        else if (r.metrics.vPre = 0 ∧ r.metrics.vPost = 0 ∧ r.metrics.spend = 0 ∧ r.metrics.defect = 0 ∧ r.metrics.authority = 0) then
          some RejectCode.vacuousZeroReceipt
        else if r.chainDigestPrev ≠ prevChainDigest ∨ ¬ chainDigestMatches r then
          some RejectCode.rejectChainDigest
        else if ¬ stateHashLinkOK prevState nextState r then
          some RejectCode.rejectStateHashLink
        else none

theorem verifyMicroRejectCode_none_of_contract
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = none := by
  rcases h with ⟨hSchema, hProfile, hObject, hNumeric, hPolicy, hDomain, hPrev, hDigest, hState⟩
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode domainLawful
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hPolicy, hDomain, hPrev, hDigest, hState]

theorem verifyMicroRejectCode_of_bad_schema
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadSchema : ¬ MicroReceipt.ValidSchema cfg r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectSchema := by
  unfold verifyMicroRejectCode
  simp [hBadSchema]

theorem verifyMicroRejectCode_of_empty_object_id
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hBadObjectId : ¬ ObjectIdValid r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectSchema := by
  unfold verifyMicroRejectCode
  simp [hSchema, hBadObjectId]

theorem verifyMicroRejectCode_of_bad_canon_profile
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hBadProfile : ¬ CanonProfilePinned cfg r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectCanonProfile := by
  unfold verifyMicroRejectCode
  simp [hSchema, hObject, hBadProfile]

theorem verifyMicroRejectCode_of_numeric_parse
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hBadParse : ¬ MetricsParseValid r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectNumericParse := by
  unfold verifyMicroRejectCode numericRejectCode
  simp [hSchema, hObject, hProfile, hBadParse]

theorem verifyMicroRejectCode_of_overflow
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hParse : MetricsParseValid r)
    (hOverflow : ¬ MetricsNoOverflow r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectOverflow := by
  unfold verifyMicroRejectCode numericRejectCode
  simp [hSchema, hObject, hProfile, hParse, hOverflow]

theorem verifyMicroRejectCode_of_policy_violation
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hNumeric : NumericValid r)
    (hBadPolicy : ¬ policyLawful r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectPolicyViolation := by
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hBadPolicy]

theorem verifyMicroRejectCode_of_spend_exceeds_balance
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hNumeric : NumericValid r)
    (hPolicy : policyLawful r)
    (hBadDomain : r.metrics.spend > r.metrics.vPre) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.spendExceedsBalance := by
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hPolicy, hBadDomain]

theorem verifyMicroRejectCode_of_vacuous_zero
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hNumeric : NumericValid r)
    (hPolicy : policyLawful r)
    (hSpend : r.metrics.spend ≤ r.metrics.vPre)
    (hVacuous : r.metrics.vPre = 0 ∧ r.metrics.vPost = 0 ∧ r.metrics.spend = 0 ∧ r.metrics.defect = 0 ∧ r.metrics.authority = 0) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.vacuousZeroReceipt := by
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hPolicy, hSpend, hVacuous]

theorem verifyMicroRejectCode_of_bad_chain_digest
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hNumeric : NumericValid r)
    (hPolicy : policyLawful r)
    (hDomain : domainLawful r)
    (hBadDigest : r.chainDigestPrev ≠ prevChainDigest ∨ ¬ chainDigestMatches r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectChainDigest := by
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode domainLawful
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hPolicy, hDomain, hBadDigest]

theorem verifyMicroRejectCode_of_bad_state_link
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hSchema : MicroReceipt.ValidSchema cfg r)
    (hObject : ObjectIdValid r)
    (hProfile : CanonProfilePinned cfg r)
    (hNumeric : NumericValid r)
    (hPolicy : policyLawful r)
    (hDomain : domainLawful r)
    (hPrev : r.chainDigestPrev = prevChainDigest)
    (hDigest : chainDigestMatches r)
    (hBadState : ¬ stateHashLinkOK prevState nextState r) :
    verifyMicroRejectCode cfg prevState nextState prevChainDigest r = some RejectCode.rejectStateHashLink := by
  rcases hNumeric with ⟨hParse, hOverflow⟩
  unfold verifyMicroRejectCode numericRejectCode domainLawful
  simp [hSchema, hObject, hProfile, hParse, hOverflow, hPolicy, hDomain, hPrev, hDigest, hBadState]

theorem rv_contract_correctness
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) :
    rv cfg prevState nextState prevChainDigest r = true ↔
      MicroReceipt.ValidSchema cfg r ∧
        CanonProfilePinned cfg r ∧
        ObjectIdValid r ∧
        NumericValid r ∧
        policyLawful r ∧
        domainLawful r ∧
        r.chainDigestPrev = prevChainDigest ∧
        chainDigestMatches r ∧
        stateHashLinkOK prevState nextState r := by
  unfold rv
  simp [microContractPred]

/-!
## Structural view lemmas (synonyms and projectors)

These lemmas expose `microContractPred` as a definitional conjunction and
provide convenient projectors for downstream use. They are intentionally kept
`simp`-friendly to reduce boilerplate in proofs.
-/

@[simp]
theorem microContractPred_iff
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) :
    microContractPred cfg prevState nextState prevChainDigest r ↔
      MicroReceipt.ValidSchema cfg r ∧
      CanonProfilePinned cfg r ∧
      ObjectIdValid r ∧
      NumericValid r ∧
      policyLawful r ∧
      domainLawful r ∧
      r.chainDigestPrev = prevChainDigest ∧
      chainDigestMatches r ∧
      stateHashLinkOK prevState nextState r := Iff.rfl

namespace microContractPred

variable {cfg : ContractConfig} {prevState nextState : StateHash}
variable {prevChainDigest : ChainDigest} {r : MicroReceipt}

theorem schema
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    MicroReceipt.ValidSchema cfg r :=
  (show _ from h).fst

theorem canon
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    CanonProfilePinned cfg r := by
  rcases h with ⟨_, hCanon, ..⟩; exact hCanon

theorem objectId
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    ObjectIdValid r := by
  rcases h with ⟨_, _, hObj, ..⟩; exact hObj

theorem numeric
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    NumericValid r := by
  rcases h with ⟨_, _, _, hNum, ..⟩; exact hNum

theorem policy
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    policyLawful r := by
  rcases h with ⟨_, _, _, _, hPol, ..⟩; exact hPol

theorem domain
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    domainLawful r := by
  rcases h with ⟨_, _, _, _, _, hDom, ..⟩; exact hDom

theorem prevDigest
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    r.chainDigestPrev = prevChainDigest := by
  rcases h with ⟨_, _, _, _, _, hPrev, ..⟩; exact hPrev

theorem digestOK
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    chainDigestMatches r := by
  rcases h with ⟨_, _, _, _, _, _, hD, ..⟩; exact hD

theorem stateLink
    (h : microContractPred cfg prevState nextState prevChainDigest r) :
    stateHashLinkOK prevState nextState r := by
  rcases h with ⟨_, _, _, _, _, _, _, _, hS⟩; exact hS

end microContractPred

theorem rv_reject_of_bad_schema
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadSchema : ¬ MicroReceipt.ValidSchema cfg r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadSchema]

theorem rv_reject_of_empty_object_id
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadObjectId : ¬ ObjectIdValid r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadObjectId]

theorem rv_reject_of_bad_canon_profile
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadProfile : ¬ CanonProfilePinned cfg r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadProfile]

theorem rv_reject_of_numeric_invalid
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadNumeric : ¬ NumericValid r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadNumeric]

theorem rv_reject_of_policy_violation
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadPolicy : ¬ policyLawful r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadPolicy]

theorem rv_reject_of_domain_invalid
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadDomain : ¬ domainLawful r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadDomain]

theorem rv_reject_of_bad_chain_digest
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadDigest : ¬ chainDigestMatches r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadDigest]

theorem rv_reject_of_bad_state_link
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hBadStateLink : ¬ stateHashLinkOK prevState nextState r) :
    rv cfg prevState nextState prevChainDigest r = false := by
  unfold rv
  simp [microContractPred, hBadStateLink]

end Coh.Contract
