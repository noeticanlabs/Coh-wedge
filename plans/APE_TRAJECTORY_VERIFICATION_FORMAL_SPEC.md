# APE Trajectory Verification Formal Specification

## 1. Core Objects

### 1.1 System State
```
S_t = (x_t, r_t, h_t, c_t, b_t, m_t)
```

| Component | Description |
|-----------|-------------|
| x_t | Operational state (task/system state the AI reasons about) |
| r_t | Receipt state (structured claim about what happened at step t) |
| h_t | History/chain state (prior receipts and lineage) |
| c_t | Constraint context (invariants, schema, verifier policies) |
| b_t | Budget state (step count, retry count, time/resource budgets) |
| m_t | Meta state (provenance, version, strategy labels, seeds) |

### 1.2 Transition Proposal
```
p_t : S_t → Ŝ_{t+1}
```
The AI proposes a transition; APE adversarially perturbs it; verifier decides admissibility.

### 1.3 Accepted Transition
```
S_t →(accept) S_{t+1}
S_t →(reject) S_t  (or explicit reject/terminal state)
```

---

## 2. Admissibility Structure (Three Levels)

### 2.1 Local State Admissibility
```
LocalOK(S_t) = true/false
```
Checks: required fields, hashes match, values valid, no single-receipt contradiction

### 2.2 Transition Admissibility
```
StepOK(S_t, S_{t+1}) = true/false
```
Checks: predecessor links match, chain origin preserved, budget updated correctly, step count advances legally

### 2.3 Trajectory Admissibility
```
TrajOK(τ) = true/false
```
Checks: no unjustified cycles, bounded retries, progress or termination, cumulative cost finite, no long-horizon drift

---

## 3. Decision Enum
```rust
pub enum Decision {
    Accept,           // Transition admitted
    Reject,          // Transition denied
    TerminalSuccess, // Verified goal state reached
    TerminalFailure, // Permanently inadmissible
    AbortBudget,     // Governance/resource limit hit
}
```

---

## 4. Reason Code Taxonomy

### 4.1 Local Integrity Failures
- `MissingSignature`
- `IntegrityMismatch`
- `EmptyRequiredField`
- `SchemaViolation`
- `FieldConsistencyFailure`

### 4.2 Chain/Continuity Failures
- `InvalidPredecessor`
- `OriginMismatch`
- `SequenceViolation`
- `ReplayDetected`
- `ChainSpliceDetected`

### 4.3 Semantic Failures
- `InvariantViolation`
- `ContradictoryClaims`
- `IntentViolation`
- `StateSemanticMismatch`

### 4.4 Trajectory Failures
- `NoProgressLoop`
- `StateCycleDetected`
- `RetryBudgetExceeded`
- `TemporalDriftDetected`
- `TrajectoryCostExceeded`

### 4.5 Resource/Governance Failures
- `StepBudgetExceeded`
- `TimeBudgetExceeded`
- `MemoryBudgetExceeded`
- `DepthLimitExceeded`

---

## 5. Trajectory Invariants

### 5.1 Local Validity
```
∀t, LocalOK(S_t) = true
```

### 5.2 Step Legality
```
∀t, StepOK(S_t, S_{t+1}) = true
```

### 5.3 Progress-or-Termination
```
nonterminal(S_t) ⇒ P(S_{t+1}) < P(S_t) OR Advance(S_t, S_{t+1})
```
Where P is a progress measure (e.g., defect mass, unverified obligations, retry debt)

### 5.4 Bounded Repetition
```
¬∃i<j with S_i ~ S_j
```
Where ~ is semantic equivalence (not raw byte equality)

### 5.5 Bounded Cumulative Cost
```
∑ cost(S_t, S_{t+1}) ≤ B_max
```

### 5.6 Chain Continuity
```
r_{t+1}.prev = digest(r_t)
```

---

## 6. APE Strategy to Invariant Mapping

| Strategy | Target Layer | Violated Invariant | Expected Code |
|----------|--------------|-------------------|----------------|
| mutation | Local | Field integrity | IntegrityMismatch |
| recombination | Step | Chain continuity | InvalidPredecessor |
| violation | Local | Invariant | InvariantViolation |
| overflow | Local | Numeric bounds | Overflow |
| contradiction | Local | Consistency | ContradictoryClaims |
| spec_gaming | Semantic | Intent | IntentViolation |
| distribution_shift | Local | Value bounds | FieldConsistencyFailure |
| temporal_drift | Trajectory | Progress | TemporalDriftDetected |
| ambiguity | Local | Schema | SchemaViolation |
| adversarial_alignment | Local | Integrity | IntegrityMismatch |
| non_termination | Trajectory | Progress | NoProgressLoop |
| livelock | Trajectory | Retry budget | RetryBudgetExceeded |
| state_explosion | Trajectory | Depth/bounds | DepthLimitExceeded |
| resource_exhaustion | Trajectory | Cost bounds | MemoryBudgetExceeded |
| parser_pathology | Local | Parsing | SchemaViolation |

---

## 7. Verification API

```rust
pub struct SystemState { /* ... */ }
pub struct Proposal { /* ... */ }
pub struct Trajectory { /* ... */ }

pub struct VerificationResult {
    pub decision: Decision,
    pub reason_code: Option<ReasonCode>,
    pub reason_detail: Option<String>,
    pub next_state: Option<SystemState>,
    pub witness: Option<Witness>,
    pub latency_us: u64,
}

fn verify_trajectory_step(
    current: &SystemState,
    proposal: &Proposal,
    prefix: &Trajectory,
) -> VerificationResult
```

---

## 8. Progress Metric (Concrete Example)

```
P(S_t) = αD_t + βU_t + γR_t + δE_t
```

| Component | Description |
|-----------|-------------|
| D_t | Unresolved defect mass |
| U_t | Remaining unverified obligations |
| R_t | Retry debt |
| E_t | Execution slack consumed |

**Demo requirement**: monotone decrease in at least one dimension, or explicit terminal state.

---

## 9. One-Paragraph Definition

> The AI/APE system is modeled as a governed transition system over operational state, receipt state, history, constraints, budget, and metadata. AI generates proposals for next-step state transitions; APE applies adversarial transformations to those proposals or their trajectory context. The verifier accepts only those transitions that preserve local validity, lawful state-to-state continuity, and global trajectory admissibility, including progress, bounded cost, and terminal safety conditions.

---

## 10. Implementation Order (Tiered)

### Tier 1 (Immediate)
- Define `SystemState`, `Proposal`, `Decision`, `ReasonCode`
- Implement `LocalOK`, `StepOK`

### Tier 2 (Short-term)
- Define `Trajectory`, `TrajOK`
- Implement progress metric, terminal conditions

### Tier 3 (Medium-term)
- Define semantic equivalence
- Implement cycle detection, replay detection, bounded cost

### Tier 4 (Validation)
- Map each APE strategy to target layer, violated invariant, expected reason code
- Generate formal attack matrix

---

*This document formalizes the trajectory admissibility verifier. The verifier approximates χ_𝒜(τ), the characteristic function of admissible trajectories.*
