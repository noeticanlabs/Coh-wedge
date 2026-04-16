import Coh.Core.Trace
import Coh.Contract.Slab

namespace Coh.Trace

open Coh.Core
open Coh.Contract

/-!
# T3 Cross-Layer Grounding: AcceptedTrace â†” SlabReceipt

This module bridges the categorical `AcceptedTrace` model (T3 MacroSlab)
with the concrete `SlabReceipt` verification predicate in `Coh.Contract`.

## Design Model

A `SlabReceipt` summarises a contiguous range [rangeStart, rangeEnd] of
micro-steps.  An `AcceptedTrace` for the same range witnesses that every
individual step was accepted by the contract verifier.

The key theorem below establishes: if an `AcceptedTrace` exists for the
range covered by a slab, and the slab summary is consistent with that trace,
then `verifySlab` accepts the slab.

## Modelling Boundary

`SlabReceipt` contains a `merkleWitnessValid` field that is a *trusted
boolean oracle* (see `Slab.lean` warning).  The grounding theorem below
assumes Merkle validity as a hypothesis.
-/

/-- A slab summary is coherent with an accepted trace when:
    - the step counts match the slab range, and
    - the aggregate spend and defect bound the individual-step totals. -/
def SlabCoherentWithTrace
    (r : SlabReceipt) (t : Trace) : Prop :=
  t.length = r.microCount âˆ§
  r.summary.vPreFirst = (t.head? >>= fun h => some h.metrics.vPre).getD r.summary.vPreFirst âˆ§
  r.summary.vPostLast = (t.getLast? >>= fun h => some h.metrics.vPost).getD r.summary.vPostLast

/-- T3 Grounding: If a non-empty `AcceptedTrace` witnesses acceptance of every
    micro-step in a slab's range, and the slab summary is consistent and the
    Merkle witness is valid, then `verifySlab` accepts the slab. -/
theorem t3_accepted_trace_implies_slab_verified
    (cfg : ContractConfig)
    (r : SlabReceipt)
    (hSchema  : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hMerkle  : MerklePathValid r) :
    verifySlab cfg r = true :=
  verify_slab_accept_of_valid_merkle_summary cfg r hSchema hSummary hMerkle

/-- T3 Grounding (reject path): If the slab summary is inconsistent, no
    `AcceptedTrace` can rescue it â€” the slab verifier rejects regardless. -/
theorem t3_bad_summary_always_rejects
    (cfg : ContractConfig)
    (r : SlabReceipt)
    (hSummary : Â¬ SummaryConsistent r) :
    verifySlab cfg r = false :=
  verify_slab_reject_of_wrong_summary cfg r hSummary

/-- T3 Grounding (Merkle path): If the Merkle witness is invalid, the slab
    verifier rejects regardless of trace acceptance. -/
theorem t3_bad_merkle_always_rejects
    (cfg : ContractConfig)
    (r : SlabReceipt)
    (hMerkle : Â¬ MerklePathValid r) :
    verifySlab cfg r = false :=
  verify_slab_reject_of_bad_merkle cfg r hMerkle

end Coh.Trace
