import Mathlib.Data.Nat.Basic
import Coh.Category.GovCat
import Coh.Contract.Micro
import Coh.Contract.RejectCode

/-!
# Coh.Bridges.ContractSoundness

Bridge from a contract-level micro verifier to a governed object `GovObj ℕ`
with a soundness law. This file encodes a minimal set of hypotheses that tie
state hashes to their numeric potentials and then constructs a `GovObj ℕ`
whose `RV` is induced by the boolean contract predicate `rv`.

**Deprecation note**: `MicroBridgeHyp` and `govObj` are maintained for backward
compatibility. New code should use `MicroBridgeHypCtx` which operates
on the contextful `GovObjCtx` category with explicit `StepCtx`.

Load-bearing statements are tagged:
- [FORMALIZED] `MicroBridgeHyp` records the necessary interface to relate
  state potential to per-receipt metrics.
- [PROVED] `MicroBridgeHyp.govObj` satisfies `rv_sound` using
  `rv_contract_correctness` and the bridge equalities.
- [FORMALIZED] `MicroBridgeHypCtx` for contextful governor objects.
-/

namespace Coh
namespace Bridges

open Coh
open Coh.Contract

/-- [FORMALIZED]
Bridge hypotheses connecting contract acceptance to state potentials. -/
structure MicroBridgeHyp where
  /-- Frozen contract configuration. -/
  cfg : ContractConfig
  /-- State potential as a function of state hash. -/
  VState : Coh.Core.StateHash → Nat
  /-- On an accepted micro step, the pre-state potential matches `vPre`. -/
  pre_eq : ∀ {prev next prevDigest} {r : MicroReceipt},
    microContractPred cfg prev next prevDigest r →
    VState prev = r.metrics.vPre
  /-- On an accepted micro step, the post-state potential matches `vPost`. -/
  post_eq : ∀ {prev next prevDigest} {r : MicroReceipt},
    microContractPred cfg prev next prevDigest r →
    VState next = r.metrics.vPost

namespace MicroBridgeHyp

variable (H : MicroBridgeHyp)

/-- [PROVED]
Construct a governed object over `ℕ` from the bridge hypotheses. The verifier
`RV` accepts exactly when the boolean contract predicate `rv` holds. -/
def govObj : GovObj Nat :=
  { X := Coh.Core.StateHash × Coh.Core.ChainDigest
  , Receipt := MicroReceipt
  , Code := RejectCode
  , canon := { profileId := "micro-gov-bridge-v1" }
  , V := fun s => H.VState s.1
  , Spend := fun r => r.metrics.spend
  , Defect := fun r => r.metrics.defect
  , RV := fun x r x' =>
      if h : rv H.cfg x.1 x'.1 x.2 r = true then
        Decision.accept
      else
        Decision.reject RejectCode.rejectPolicyViolation
  , rv_sound := by
      intro x r x' hAcc
      classical
      -- Either the boolean predicate is true or false; acceptance forces true.
      by_cases hv : rv H.cfg x.1 x'.1 x.2 r = true
      · -- From `rv = true`, we obtain the full contract predicate
        have hPred : microContractPred H.cfg x.1 x'.1 x.2 r :=
          (rv_contract_correctness (cfg := H.cfg)
            (prevState := x.1) (nextState := x'.1)
            (prevChainDigest := x.2) (r := r)).mp hv
        -- Extract policy inequality and bridge equalities for V
        rcases hPred with ⟨hSchema, hProfile, hObject, hNumeric, hPolicy, hPrevEq, hDigest, hState⟩
        have hPre : H.VState x.1 = r.metrics.vPre := H.pre_eq (r := r)
          (by
            -- reassemble the tuple because we consumed it above
            exact And.intro hSchema (And.intro hProfile (And.intro hObject
              (And.intro hNumeric (And.intro hPolicy (And.intro hPrevEq
                (And.intro hDigest hState)))))))
        have hPost : H.VState x'.1 = r.metrics.vPost := H.post_eq (r := r)
          (by
            exact And.intro hSchema (And.intro hProfile (And.intro hObject
              (And.intro hNumeric (And.intro hPolicy (And.intro hPrevEq
                (And.intro hDigest hState)))))))
        -- Goal: V x' + spend ≤ V x + defect
        -- Rewrite via bridge equalities and apply policy inequality
        -- hPolicy : vPost + spend ≤ vPre + defect
        simpa [hPre, hPost] using hPolicy
      · -- Contradiction: acceptance cannot arise if rv = false
        -- Evaluate the `if` and derive an impossibility
        simp [govObj, hv] at hAcc
  }

end MicroBridgeHyp

/-- [FORMALIZED]
Contextual bridge hypotheses for StepCtx-dependent verification. This extends
`MicroBridgeHyp` to work with the contextful `GovObjCtx` category. -/
structure MicroBridgeHypCtx where
  /-- Frozen contract configuration. -/
  cfg : ContractConfig
  /-- State potential as a function of state hash. -/
  VState : Coh.Core.StateHash → Nat
  /-- On an accepted micro step, the pre-state potential matches `vPre`. -/
  pre_eq : ∀ {prev next prevDigest} {r : MicroReceipt},
    microContractPred cfg prev next prevDigest r →
    VState prev = r.metrics.vPre
  /-- On an accepted micro step, the post-state potential matches `vPost`. -/
  post_eq : ∀ {prev next prevDigest} {r : MicroReceipt},
    microContractPred cfg prev next prevDigest r →
    VState next = r.metrics.vPost

namespace MicroBridgeHypCtx

variable (H : MicroBridgeHypCtx)

/-- [PROVED]
Construct a contextful governed object from contextual bridge hypotheses.
The verifier takes explicit `StepCtx` carrying the previous chain digest. -/
def govObjCtx : GovObjCtx Nat :=
  { X := Coh.Core.StateHash × Coh.Core.ChainDigest
  , Receipt := MicroReceipt
  , Code := RejectCode
  , canon := { profileId := "micro-gov-bridge-ctx-v1" }
  , V := fun s => H.VState s.1
  , Spend := fun r => r.metrics.spend
  , Defect := fun r => r.metrics.defect
  , RV := fun c x r x' =>
      if h : rv H.cfg x.1 x'.1 (Coh.Contract.Digest.repr c.prevChainDigest).toString r = true then
        Decision.accept
      else
        Decision.reject RejectCode.rejectPolicyViolation
  , rv_sound := by
      intro c x r x' hAcc
      classical
      by_cases hv : rv H.cfg x.1 x'.1 (Coh.Contract.Digest.repr c.prevChainDigest).toString r = true
      · have hPred : microContractPred H.cfg x.1 x'.1 (Coh.Contract.Digest.repr c.prevChainDigest).toString r :=
          (rv_contract_correctness (cfg := H.cfg)
            (prevState := x.1) (nextState := x'.1)
            (prevChainDigest := (Coh.Contract.Digest.repr c.prevChainDigest).toString) (r := r)).mp hv
        rcases hPred with ⟨hSchema, hProfile, hObject, hNumeric, hPolicy, hPrevEq, hDigest, hState⟩
        have hPre : H.VState x.1 = r.metrics.vPre := H.pre_eq (r := r) (by exact hPred)
        have hPost : H.VState x'.1 = r.metrics.vPost := H.post_eq (r := r) (by exact hPred)
        simpa [hPre, hPost] using hPolicy
      · simp [govObjCtx, hv] at hAcc }

end MicroBridgeHypCtx

end Bridges
end Coh
