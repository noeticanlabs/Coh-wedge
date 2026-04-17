import Coh.Contract.RejectCode
import Coh.Contract.Micro
import Coh.Contract.Slab
import Coh.Category.GovCat

/-!
# Coh.Contract.VerifierBoundary

Deterministic boundary verifier interfaces returning `Coh.Decision RejectCode`.
These are thin wrappers around the existing Option-returning reject-code
functions, lifting them into a canonical Decision type suitable for the category
layer and bridge constructions.
-/

namespace Coh.Contract

open Coh

/- Deterministic decision from Option RejectCode -/
def decisionOfOption (o : Option RejectCode) : Decision RejectCode :=
  match o with
  | none      => Decision.accept
  | some code => Decision.reject code

/-!
## Micro (boundary) verifier as Decision

Converts `verifyMicroRejectCode = none` into `Decision.accept` and `some code`
into `Decision.reject code`.
-/
def verifyMicroDecision
    (cfg : ContractConfig)
    (prevState nextState : Coh.Core.StateHash)
    (prevChainDigest : Coh.Core.ChainDigest)
    (r : MicroReceipt) : Decision RejectCode :=
  decisionOfOption (verifyMicroRejectCode cfg prevState nextState prevChainDigest r)

/-!
## Slab (boundary) verifier as Decision

Similarly wraps the slab verifier reject-code into a deterministic Decision.
-/
def verifySlabDecision
    (cfg : ContractConfig)
    (r : Coh.Contract.SlabReceipt) : Decision RejectCode :=
  decisionOfOption (verifySlabRejectCode cfg r)

end Coh.Contract
