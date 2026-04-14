import Coh.Contract.Micro

namespace Coh.Core

open Coh.Contract

abbrev LinkedStep
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Prop :=
  r.chainDigestPrev = prevChainDigest ∧ stateHashLinkOK prevState nextState r

abbrev AcceptedStep
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt) : Prop :=
  rv cfg prevState nextState prevChainDigest r = true

theorem accepted_step_implies_chain_digest_correct
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hAccepted : AcceptedStep cfg prevState nextState prevChainDigest r) :
    r.chainDigestNext = digestUpdate prevChainDigest r.canonicalPayload := by
  have hContract := (rv_contract_correctness cfg prevState nextState prevChainDigest r).mp hAccepted
  rcases hContract with ⟨_, _, _, _, hPrevDigest, hDigest, _⟩
  unfold chainDigestMatches at hDigest
  simpa [hPrevDigest] using hDigest

theorem accepted_step_implies_state_hash_link
    (cfg : ContractConfig)
    (prevState nextState : StateHash)
    (prevChainDigest : ChainDigest)
    (r : MicroReceipt)
    (hAccepted : AcceptedStep cfg prevState nextState prevChainDigest r) :
    stateHashLinkOK prevState nextState r := by
  have hContract := (rv_contract_correctness cfg prevState nextState prevChainDigest r).mp hAccepted
  rcases hContract with ⟨_, _, _, _, _, _, hLink⟩
  exact hLink

end Coh.Core
