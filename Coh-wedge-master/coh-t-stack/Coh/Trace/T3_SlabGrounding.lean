import Coh.Core.Trace
import Coh.Contract.Slab

namespace Coh.Trace

open Coh.Core
open Coh.Contract

/-!
# T3 Cross-Layer Grounding: AcceptedTrace ↔ SlabReceipt

This module bridges the core `AcceptedTrace` model with the concrete
`SlabReceipt` verifier in `Coh.Contract.Slab`.

Model boundary note: `merkleWitnessValid` is a trusted boolean oracle
populated by the Rust verifier (see `Slab.lean`).
-/

/-- Coherence ledger linking a slab to a concrete accepted trace. -/
def SlabCoherentWithTrace (r : SlabReceipt) (t : Trace) : Prop :=
  -- Range/size agreement
  t.length = r.microCount ∧
  r.rangeStart ≤ r.rangeEnd ∧
  r.microCount = r.rangeEnd - r.rangeStart + 1 ∧
  -- Head/last metric anchors
  r.summary.vPreFirst = (t.head? >>= fun h => some h.metrics.vPre).getD r.summary.vPreFirst ∧
  r.summary.vPostLast = (t.getLast? >>= fun h => some h.metrics.vPost).getD r.summary.vPostLast ∧
  -- Exact aggregates (totals) used for telescoping
  r.summary.totalSpend = Coh.Core.totalSpend t ∧
  r.summary.totalDefect = Coh.Core.totalDefect t ∧
  -- Pre-validated overflow guard carried by the summarizer
  Coh.Contract.SummaryNoOverflow r ∧
  -- Metric continuity is required for telescoping to apply across the slab
  Coh.Core.MetricsContinuous t

/--- T3 Grounding: Direct corollary when a consistent summary and valid Merkle
    witness are already available. -/
theorem t3_accepted_trace_implies_slab_verified
    (cfg : ContractConfig) (r : SlabReceipt)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hSummary : SummaryConsistent r)
    (hMerkle  : MerklePathValid r) :
    verifySlab cfg r = true :=
  Coh.Contract.verify_slab_accept_of_valid_merkle_summary cfg r hSchema hSummary hMerkle

/-- T3 Grounding (reject path): If the slab summary is inconsistent, no
    `AcceptedTrace` can rescue it — the slab verifier rejects regardless. -/
theorem t3_bad_summary_always_rejects
    (cfg : ContractConfig)
    (r : SlabReceipt)
    (hSummary : ¬ SummaryConsistent r) :
    verifySlab cfg r = false :=
  verify_slab_reject_of_wrong_summary cfg r hSummary

/-- T3 Grounding (Merkle path): If the Merkle witness is invalid, the slab
    verifier rejects regardless of trace acceptance. -/
theorem t3_bad_merkle_always_rejects
    (cfg : ContractConfig)
    (r : SlabReceipt)
    (hMerkle : ¬ MerklePathValid r) :
    verifySlab cfg r = false :=
  verify_slab_reject_of_bad_merkle cfg r hMerkle

end Coh.Trace


/-- Soundness: If a slab correctly summarizes an accepted trace, then the slab is verifiable. --/
theorem slab_soundness_theorem
    (cfg : ContractConfig) (r : SlabReceipt) (t : Trace)
    -- The trace semantics witnessed by the core verifier over the same range
    (hTrace : Coh.Core.AcceptedTrace cfg r.rangeStart r.stateHashFirst r.stateHashLast r.chainDigestPrev t)
    -- Coherence ledger carrying counts, anchors, exact aggregates, overflow guard, and continuity
    (hCoherent : SlabCoherentWithTrace r t)
    (hSchema : SlabReceipt.ValidSchema cfg r)
    (hMerkle : MerklePathValid r) :
    verifySlab cfg r = true := by
  -- Unpack coherence obligations (now 9 fields with continuity)
  rcases hCoherent with
    ⟨hLen, hRange, hCountMatch, hPreEq, hPostEq, hSpendEq, hDefectEq, hNoOverflow, hCont⟩
  -- Build SummaryPolicyLawful via chain telescoping (requires continuity)
  have hTel := Coh.Core.chain_telescoping_theorem (cfg := cfg) (t := t) hTrace hCont
  -- Rewrite with coherence equalities to target the slab summary fields
  -- Head and last metrics align via hPreEq/hPostEq, totals via hSpendEq/hDefectEq
  have hPolicy : Coh.Contract.SummaryPolicyLawful r := by
    unfold Coh.Contract.SummaryPolicyLawful
    -- Telescoping gives: vPost_last + totalSpend ≤ vPre_first + totalDefect
    -- Substitute anchored values from coherence
    simpa [hPreEq, hPostEq, hSpendEq, hDefectEq]
      using hTel
  -- Assemble SummaryConsistent from coherence + derived policy law
  have hSummary : Coh.Contract.SummaryConsistent r := by
    refine And.intro ?hNonempty (And.intro hRange (And.intro hCountMatch (And.intro hNoOverflow hPolicy)))
    -- Nonempty from positive microCount (derived from range count + 1)
    have : 0 < r.microCount := by
      have : 1 ≤ r.microCount := by
        simpa [hCountMatch] using Nat.le_of_lt_succ (Nat.lt_succ_self (r.rangeEnd - r.rangeStart))
      exact Nat.lt_of_le_of_ne this (by decide)
    exact this
  -- Conclude via the envelope + Merkle acceptance theorem
  exact Coh.Contract.verify_slab_accept_of_valid_merkle_summary cfg r hSchema hSummary hMerkle
