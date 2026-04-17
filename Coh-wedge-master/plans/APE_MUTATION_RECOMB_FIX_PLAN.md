# APE Mutation/Recombination Fix Plan

## System Analysis (State Space View)

```
S = (X, R, C, Π, H)
```

| Component | Description | APE Target |
|-----------|------------|------------|
| X | Physical/logical state | No |
| R | Receipt state (ledger) | Yes - mutation |
| C | Constraints/invariants | Yes - violation/overflow/contradiction |
| Π | Policy/AI controller | No |
| H | History/trace | Yes - recombination |

**Gap identified**: Verifier strong on C (invariants) but weak on R integrity and H structure.

## Current State
- 60% overall rejection (150/250)
- 100% rejection for violation, overflow, contradiction  
- **0% rejection for mutation and recombination** (both escaping 50/50)

## Target
> **Reject all security-relevant mutation/recombination attacks, while allowing harmless variation.**

This keeps the verifier honest instead of paranoid.

## Phase 1 — Define Attack Subtypes

### Mutation buckets
| Subtype | Description | Expected |
|--------|------------|-----------|
| cosmetic | whitespace, formatting, note text, non-semantic metadata | may pass |
| integrity | receipt fields, amounts, IDs, timestamps, hashes, predecessor refs | **must fail** |
| consistency | change one field without updating dependent fields | **must fail** |
| provenance | alter issuer/origin/chain identity | **must fail** |

### Recombination buckets
| Subtype | Description | Expected |
|--------|------------|-----------|
| benign | combines fragments preserving valid lineage | may pass |
| chain_splice | inserts valid fragment into wrong history | **must fail** |
| cross_origin | mixes fragments from different chains/sessions | **must fail** |
| sequence_violation | valid parts, invalid order | **must fail** |
| hash_link_break | predecessor references don't match | **must fail** |

## Phase 2 — Instrument Escapes

For every escaped candidate, record:
- `strategy`, `attack_subtype`, `changed_fields`, `verifier_decision_path`

Build triage table to separate benign vs real misses.

## Phase 3 — Add Verifier Gates

### Gate 1: Receipt Canonicalization
- Canonical serialize before verify
- Normalize field ordering
- Hash canonical form
- Catches field tampering

### Gate 2: Cross-field Consistency
- Derived fields match source
- IDs align with referenced objects
- Totals/counts/bounds match transformed content

### Gate 3: Provenance/Lineage Validation
- Origin matches chain origin
- Predecessor exists and matches expected prior digest
- Chain position valid
- Issuer/session/domain coherent

### Gate 4: Whole-chain Verification
- Continuity, uniqueness, no duplicates/skips
- No valid local segment in invalid global history

## Phase 4 — Strengthen APE Generators

### Mutation upgrades
- `tamper_payload_only`: modify payload without updating digest
- `tamper_digest_only`: modify digest without payload  
- `tamper_link_ref`: change predecessor ref
- `tamper_origin`: change issuer/origin
- `tamper_dependent_field`: change one member of dependent pair

### Recombination upgrades
- `splice_foreign_fragment`: from different seed/chain
- `reorder_links`: valid fragments, invalid order
- `duplicate_link`: repeat valid link
- `drop_middle_link`: remove and reconnect improperly
- `graft_mismatched_suffix`: valid prefix onto unrelated suffix

## Phase 5 — Explicit Failure Reasons

Add to verifier result:
- `IntegrityMismatch`
- `CrossFieldInconsistency`
- `InvalidPredecessor`
- `ChainOriginMismatch`
- `SequenceViolation`
- `ContradictoryClaims`
- `BoundsExceeded`

## Implementation Order

### Sprint 1
- [ ] Add subtypes to CandidateMetadata
- [ ] Instrument escaped cases
- [ ] Build escape triage table

### Sprint 2
- [ ] Implement canonicalization + integrity digest
- [ ] Run mutation demo - expect sharp rise

### Sprint 3
- [ ] Implement lineage/provenance/sequence checks  
- [ ] Run recombination demo - expect sharp rise

### Sprint 4
- [ ] Add explicit failure reason codes
- [ ] Update demo table with reason distribution

## Target Demo Output

| Strategy      | Generated | Rejected | Escaped | Top Reason          |
| ------------- | --------: | -------: | ------: | ------------------- |
| mutation      |       100 |       ~97 |       ~3 | IntegrityMismatch   |
| recombination |       100 |       ~95 |       ~5 | InvalidPredecessor  |
| violation     |       100 |      100 |       0 | InvariantViolation  |
| overflow      |       100 |      100 |       0 | BoundsExceeded      |
| contradiction |       100 |      100 |       0 | ContradictoryClaims |

## Safe Pitch During Fix

> "APE is now partitioning verifier performance by semantic attack class. The verifier already fully rejects invariant, overflow, and contradiction attacks. Mutation and recombination results are driving the current integrity and lineage hardening pass."

## Formal Summary (Investor-safe)

> "We model AI execution as a constrained state transition system over state, receipt history, and invariants. APE generates adversarial perturbations of the receipt and history components, and the verifier enforces admissibility by rejecting any state outside the valid constraint region."

The game: APE finds holes in the decision boundary, verifier patches them.