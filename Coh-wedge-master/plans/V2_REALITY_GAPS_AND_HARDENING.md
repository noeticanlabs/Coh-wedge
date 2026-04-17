# V2 Reality Gaps and System Hardening

> Analysis of real-world attack surface and blind spots, plus a three-extension roadmap to move from "strong prototype" to "serious system architecture"

## Executive Summary

The current Coh system answers the question: *"Can this transition exist?"*

It does NOT yet answer:

- Should this transition exist?
- What happens over sequences of valid transitions?
- Who defines what "valid" means?
- What if the state representation is wrong?
- What if users fight the system?

This document analyzes 11 gaps discovered in real-world deployment consideration and proposes three targeted extensions that address the highest-impact issues without bloating the core wedge.

---

## Part 1: Gap Analysis

### Gap 1: The "Valid but Undesirable" Problem

**Problem**: Your system guarantees invalid transitions are inexpressible. It does NOT guarantee desirable outcomes.

**Example**: An agent that closes all tickets to reduce backlog submits valid transitions, budget is OK, policy allows it—but the outcome is operationally wrong.

**Root cause**: Constraint layer (you have) != objective layer (you don't).

**Fix direction**: Add objective layer with priority weighting, goal consistency checks, or human-in-the-loop override.

---

### Gap 2: Policy Is the Real Attack Surface

**Problem**: Whoever controls the policy hash controls reality. If policy is misconfigured, maliciously altered, or too permissive, invalid behavior becomes "valid by definition."

**Missing**: Policy governance layer

- policy versioning
- policy signing
- policy audit trails
- policy diff verification

---

### Gap 3: Time / Ordering Attacks

**Problem**: Verifying single transitions doesn't catch sequences of valid transitions that together cause catastrophic outcomes.

**Example**: Step 1 valid, Step 2 valid, Step 3 valid—but together: catastrophic

**Missing**: Temporal coherence / sequence constraints

**Fix ideas**: rolling invariants, cumulative budget checks, no-regret windows, state drift limits

---

### Gap 4: State Representation Risk

**Problem**: Your system assumes state is correctly represented. But if state is incomplete, abstracted poorly, or projection is lossy, the verifier approves based on false reality.

**Root cause**: Your own projection law (information loss through abstraction) has no formalized defense.

**Missing**: State integrity guarantees

- validation of state inputs
- confidence bounds
- "unknown state" handling
- degraded-mode verification

---

### Gap 5: Performance vs Enforcement Tradeoff

**Problem**: Every action requires verification (hashing + checks). If too slow, users will bypass, weaken, or disable enforcement.

**Missing**: Tiered verification modes

- strict (full checks)
- fast path (cached / partial)
- async audit (post-check)

---

### Gap 6: The "Bypass Layer" Problem

**Problem**: Even perfect verification can be bypassed through shadow execution—direct API calls, side channels.

**Missing**: Enforcement boundary guarantees

- is verification mandatory?
- can it be bypassed?
- what enforces its use?

---

### Gap 7: Composability Between Systems

**Problem**: Multiple agents, multiple verifiers, different policies colliding—what happens?

**Missing**: Inter-verifier coherence

- cross-system receipts
- shared policy domains
- conflict resolution

---

### Gap 8: Over-Rejection (Adoption Killer)

**Problem**: If system rejects too much, users say "it blocks everything" and abandon it.

**Missing**: Graceful rejection strategies

- explainable reject codes (EXIST)
- fallback paths
- suggestion engine (what WOULD pass)

---

### Gap 9: Human Override Layer

**Problem**: Users will ask "Can I override it?" Answer no = no adoption. Answer yes = need formal override mechanism.

**Missing**: Governed override

- override receipts
- elevated permissions
- traceable exceptions

---

### Gap 10: Economic Layer (Uncovered Opportunity)

**Problem**: You have spend/defect/budget but haven't mapped to real cost/value.

**Missing**: Real cost / value mapping

- compute cost
- API usage cost
- operational cost
- risk cost

**Opportunity**: AI execution accounting layer (VERY fundable)

---

### Gap 11: The Hidden Killer Feature

**Problem**: You built deterministic replay but haven't pushed it to full implication.

**Opportunity**: forensic debugging, regulatory compliance, simulation before execution, "what-if" analysis

---

## Part 2: Three High-Leverage Extensions

### Extension 1: Transition Contract (Explicit)

Make the contract layer the centerpiece. This is the highest-value addition.

**Scope**:

```
TransitionContract {
  constraint_satisfied: bool
  objective_satisfied: bool | null  // NEW
  sequence_valid: bool               // NEW (sequence guard)
  override_applied: bool             // NEW (governance)
  metadata: ContractMetadata
}
```

**Implementation**:

1. Add `objective_satisfied` field (nullable, null = not checked)
2. Add sequence validation layer
3. Add override flag to receipts
4. Document when objective layer applies

**Impact**: Moves from "can this exist?" to "should this exist?"

---

### Extension 2: Policy Governance Layer

Policies become versioned, signed, receipted artifacts.

**Scope**:

```
PolicyReceipt {
  policy_id: String
  version: u32
  content_hash: Digest
  signature: Signature
  previous_receipt_hash: Digest  // chain of policy
  valid_from: Timestamp
  metadata: PolicyMetadata
}
```

**Implementation**:

1. Define `PolicyReceipt` type in Lean
2. Add policy chain verification to verifier
3. Allow policy transitions (policy changes must be valid)
4. Add policy diff verification

**Impact**: "Policy changes are themselves verified transitions"

---

### Extension 3: Sequence / Temporal Guard

Even a simple version prevents temporal attacks.

**Scope**:

```
SequenceGuard {
  max_state_drift: u128      // maximum state change per window
  max_cumulative_spend: u128  // rolling spend limit
  window_length: u32       // steps in window
  invariant_checks: []       // custom rolling invariants
}
```

**Implementation**:

1. Add `SequenceGuard` config type
2. Implement rolling accumulator
3. Add state-drift check to verification
4. Add cumulative-spend check

**Impact**: Sequences of valid transitions are also validated

---

## Part 3: Implementation Roadmap

### Phase 1: Transition Contract (highest leverage)

- [ ] Add `TransitionContract` type to Lean core
- [ ] Add `objective_satisfied: Option<bool>` field
- [ ] Document objective layer semantics
- [ ] Add example objectives (priority, goal check)

### Phase 2: Policy Governance (attack surface fix)

- [ ] Define `PolicyReceipt` type
- [ ] Implement policy chain verification
- [ ] Add policy versioning to config
- [ ] Add policy signing interface
- [ ] Document policy update flow

### Phase 3: Sequence Guard (temporal attack fix)

- [ ] Define `SequenceGuard` config
- [ ] Implement rolling accumulator
- [ ] Add state-drift check
- [ ] Add cumulative spend check
- [ ] Document rolling invariant semantics

### Phase 4: Tiered Verification (performance fix)

- [ ] Define verification modes (strict/fast/async)
- [ ] Implement fast-path optimization
- [ ] Add async audit queue
- [ ] Document performance characteristics

---

## Part 4: Gap Coverage Matrix

| Gap | Covered by Extension | Priority |
|-----|-----------------|----------|
| Valid but undesirable | #1 Transition Contract | P0 |
| Policy attack surface | #2 Policy Governance | P0 |
| Time/ordering attacks | #3 Sequence Guard | P0 |
| State representation | #1 + explicit input validation | P1 |
| Performance tradeoff | #4 Tiered Verification | P1 |
| Bypass layer | Enforcement boundary doc | P1 |
| Composability | (future phase) | P2 |
| Over-rejection | Suggestion engine | P1 |
| Human override | #1 Transition Contract | P0 |
| Economic layer | (future phase) | P2 |
| Deterministic replay | Already available | N/A |

---

## Part 5: Risk Assessment

### High Risk (address now)

1. **Policy as attack surface**: Malicious policy = system compromised
2. **Sequences invalid**: Catastrophic outcomes via valid sequence
3. **Bypass**: Verification is optional

### Medium Risk (Phase 2)

1. **Performance**: Users disable to workaround
2. **State representation**: False positives
3. **Override**: Ungoverned override

### Lower Risk (Phase 3+)

1. **Composability**
2. **Economic layer**
3. **Over-rejection**

---

## Appendix: Related Documents

- [`SECURITY_MODEL.md`](SECURITY_MODEL.md) - Current threat model (does NOT address these gaps)
- [`OPERATIONAL_ARCHITECTURE.md`](OPERATIONAL_ARCHITECTURE.md) - Current deployment (includes performance considerations)
- [`CONTRACT_LAYER_FREEZE_PLAN.md`](CONTRACT_LAYER_FREEZE_PLAN.md) - Current contract boundary

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial document from gap analysis |