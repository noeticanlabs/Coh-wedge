# PhaseLoom Integration Plan

> PhaseLoom: receipt-grounded adaptive memory for NPE (Non-deterministic Proposal Engine) strategy selection.

---

## 1. Role Separation

The PhaseLoom architecture maintains strict role boundaries:

| Component | Responsibility | Access |
|-----------|---------------|-------|
| `coh-core` | Verifier/admission kernel | Deterministic RV predicate, policy enforcement |
| `coh-genesis` | NPE, Genesis metrics, proof/patch generation | Proposal generation, candidate ranking |
| `phaseloom` | Receipt-grounded memory/adaptation layer | Learn from receipts, bias strategy weights |

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   NPE      │────▶│   Coh     │────▶│   Lean    │────▶│  PhaseLoom │
│ Proposes   │     │   Gates   │     │  Verifies │     │  Learns   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       ▲                                                            │
       │                                                            │
       └────────────────────────────────────────────────────────────┘
                    Bias future proposals
```

**Safety Invariant**: PhaseLoom may bias future proposals, but may **never** bypass Coh/RV/Lean verification.

---

## 2. Minimal PhaseLoomLite State

Located at [`coh-node/crates/coh-genesis/src/phaseloom_lite.rs`](coh-node/crates/coh-genesis/src/phaseloom_lite.rs):

```rust
/// Strategy weight vector indexed by step_type/strategy_class
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct StrategyWeights(pub HashMap<String, f64>);

/// PhaseLoomLite state: adaptive memory for NPE
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PhaseLoomState {
    /// Strategy weight vector (bias toward proven useful outcomes)
    pub strategy_weights: StrategyWeights,
    /// Structural curvature (accumulated rejection stress)
    pub curvature: u128,
    /// Remaining budget ( thermodynamic work capacity)
    pub budget: u128,
    /// Intrinsic machine time (step counter, not wall clock)
    pub tau: u64,
}

impl Default for PhaseLoomState {
    fn default() -> Self {
        Self {
            strategy_weights: StrategyWeights::default(),
            curvature: 0,
            budget: 100_000,  // Configurable
            tau: 0,
        }
    }
}
```

### State Invariants

| Field | Constraint | Meaning |
|-------|------------|---------|
| `curvature` | `curvature ≥ 0` | Accumulated rejection penalty |
| `budget` | `budget ≥ 0` | Thermodynamic work budget |
| `tau` | `tau ≥ 0` | Monotonically increasing step index |
| `strategy_weights` | Weights normalized to sum = 1.0 | Probability distribution over strategies |

---

## 3. Receipt Input Format

PhaseLoom consumes summary receipts from Coh verification outcomes:

```rust
/// Boundary receipt summary consumed by PhaseLoom
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BoundaryReceiptSummary {
    /// Domain (e.g., "code", "test", "docs")
    pub domain: String,
    /// Target (e.g., "function foo", "module bar")
    pub target: String,
    /// Strategy class used (e.g., "synthesize", "refine", "debug")
    pub strategy_class: String,
    /// Wildness parameter (0.0 = conservative, 1.0 = aggressive)
    pub wildness: f64,
    /// Genesis margin: M(g') + C(p) - M(g) - D(p)
    pub genesis_margin: i128,
    /// Coherence margin: V_post + spend - V_pre - defect
    pub coherence_margin: i128,
    /// First failure reason if rejected
    pub first_failure: String,
    /// Outcome: "accepted", "rejected", "erroneous"
    pub outcome: String,
    /// Accepted: true/false
    pub accepted: bool,
    /// Novelty score (0.0 = repeat, 1.0 = novel)
    pub novelty: f64,
    /// Receipt hash for audit trail
    pub receipt_hash: String,
}
```

---

## 4. Update Law

PhaseLoom updates strategy weights based on acceptance and useful outcomes:

### Strategy Weight Update

```
w_{c,n+1} = Normalize( w_{c,n} + η × R_c - ρ × F_c )
```

Where:
- `w_{c,n}` = weight for strategy class c at time n
- `η` = learning rate (positive reward for acceptance)
- `R_c` = reward signal (1.0 if accepted, 0.5 if useful but not optimal)
- `ρ` = curvature penalty coefficient
- `F_c` = failure count for strategy class c

### Curvature Update

```
C_{n+1} = C_n + Σ (rejected outcomes)
```

### Budget Update

```
B_{n+1} = B_n - spend (if accepted)
B_{n+1} = B_n - penalty (if rejected)
```

### Implementation

```rust
impl PhaseLoomState {
    /// Process a boundary receipt and update internal state
    pub fn ingest(&mut self, receipt: &BoundaryReceiptSummary) {
        // Update step counter
        self.tau = self.tau.saturating_add(1);

        // Curvature accumulation (rejection penalty)
        if !receipt.accepted {
            self.curvature = self.curvature.saturating_add(1);
        }

        // Budget burn (work consumption)
        let spend = if receipt.accepted { 10 } else { 50 };
        self.budget = self.budget.saturating_sub(spend);

        // Strategy weight update
        let class = &receipt.strategy_class;
        let reward = if receipt.accepted { 0.1 } else { 0.0 };
        let penalty = if !receipt.accepted { 0.05 } else { 0.0 };

        self.strategy_weights.0.entry(class.clone())
            .and_modify(|w| *w = (*w + reward - penalty).clamp(0.0, 1.0))
            .or_insert(reward);

        // Normalize weights to probability distribution
        self.strategy_weights.normalize();
    }

    /// Sample a strategy based on current weights
    pub fn sample_strategy(&self, rng: &mut dyn RngCore) -> String {
        // Weighted random sampling from strategy_weights
        // ...
    }
}
```

---

## 5. Safety Rules

### Rule 1: Verification Bypass Forbidden

> PhaseLoom may change proposal probabilities, but it may never change admission rules.

> PhaseLoom is **advisory**. Coh remains **authoritative**.

Implementation: PhaseLoom output is **advisory only**, never gates acceptance.

### Rule 2: Deterministic Verification Priority

> If Coh verification rejects, PhaseLoom state updates on rejection but cannot override.

### Rule 2: Deterministic Verification Priority

> If Coh verification rejects, PhaseLoom state updates on rejection but cannot override.

### Rule 3: Bounded Curvature

> Excessive curvature accumulation triggers circuit-breaker (pause adaptive learning).

```rust
impl PhaseLoomState {
    pub fn is_circuit_broken(&self) -> bool {
        self.curvature > 10_000 || self.budget == 0
    }
}
```

### Rule 4: Audit Trail

> Every PhaseLoom state update must be traceable to a receipt hash.

---

## 6. API Surface

### Public Functions

```rust
/// Initialize PhaseLoom state
pub fn phaseloom_init(config: &PhaseLoomConfig) -> PhaseLoomState;

/// Ingest a boundary receipt
pub fn phaseloom_ingest(state: &mut PhaseLoomState, receipt: &BoundaryReceiptSummary);

/// Sample next strategy (advisory, never bypasses verification)
pub fn phaseloom_sample(state: &PhaseLoomState, rng: &mut dyn RngCore) -> String;

/// Check circuit breaker
pub fn phaseloom_circuit_broken(state: &PhaseLoomState) -> bool;

/// Serialize state for persistence
pub fn phaseloom_serialize(state: &PhaseLoomState) -> Vec<u8>;
```

---

## 7. Gap Analysis (for math review)

| Issue | Severity | Status |
|------|----------|--------|
| Convergence theorem assumptions not explicitly stated | HIGH | Needs review |
| "Strictly halts" stronger than justified | MEDIUM | Recommend bounded convergence |
| τ monotonicity assumes finite total work | MEDIUM | Needs bounded time definition |
| KL property not proven for this system | HIGH | Requires existence proof |
| Oplax energy inequality not explicit | HIGH | Needs formula in Ψ, δ_proj |

---

## 8. Math Review Findings

### Issue 1: Convergence Theorem Assumptions (Theorems 11-14)

**Problem**: The spec states "the system strictly halts" but doesn't justify the conditions required.

**Required assumptions** (not stated):

1. **Closed/convex admissible set** $\mathcal{X}_{\mathrm{adm}}$ - Needed for weak compactness
2. **Existence of absolutely continuous trajectories** - Flow solution requires PDE/ODE existence
3. **Descent compatibility** - Energy must decrease along trajectories
4. **KL property** - Needed for convergence rate
5. **Boundedness** - Precompactness requires bounded sublevel sets

**Recommendation**: Add explicit hypotheses section to each theorem.

### Issue 2: "Strictly Halts" Claim

**Problem**: The spec claims "strictly halts" which implies finite convergence time. This is stronger than what's proven.

**Current state**: Precompactness guarantees cluster points exist, but not finite convergence.

**Recommendation**: Replace with "finite-length convergence under KL/descent assumptions" or "subsequential convergence to critical set."

### Issue 3: τ (Intrinsic Time) Monotonicity

**Problem**: τ is defined as strictly increasing, but convergence of τ → ∞ needs justification.

**Requirements**:

- Finite total lawful work budget
- Bounded step computation cost per transition
- Or redefine τ as index variable (not time-dependent)

**Recommendation**: Treat τ as step index (natural numbers), not continuous time.

### Issue 4: KL Property Not Proven

**Problem**: Kurdyka-Łojasiewicz convergence requires the objective to satisfy KL inequality.

**Status**: Not established for PhaseLoom energy functional $\mathcal{E}$.

**Recommendation**: Either (a) prove KL for V(x) or (b) assume weakened convergence ("subsequential limit exists").

### Issue 5: Oplax Energy Inequality Not Explicit

**Problem**: The spec references "categorical energy inequality" but doesn't give explicit formula.

**Should include**:

```
E(x') ≤ E(x) + δ_proj
```

Where:

- E is the extended energy functional
- δ_proj is the projection slack (coarse-graining error)

**Recommendation**: Define as:

```
Ψ(x', C') + I_K(x', C') ≤ Ψ(x, C) + δ_proj(x, x')
```

### Summary Table

| Theorem | Gap | Recommended Fix |
|---------|-----|-----------------|
| Theorem 11 | Assumptions missing | Add:closed convex X_adm, flow existence |
| Theorem 12 | Coercivity not checked | Add:coercive V on X_adm |
| Theorem 13 | Compact embedding | Specify Banach space |
| Theorem 14 | Reduced state | Clarify topology |
| Overall | "Strictly halts" | Replace with bounded convergence |

---

## 8. File Locations

| File | Location |
|------|----------|
| PhaseLoomLite implementation | `coh-node/crates/coh-genesis/src/phaseloom_lite.rs` |
| Module entry | `coh-node/crates/coh-genesis/src/lib.rs` |
| Integration plan | This file |
| Math specification | User-provided PhaseLoom spec |

---

## 9. References

- [PhaseLoom Complete Canonical Framework](./PHASELOOM_SPEC.md) (external)
- [coh-core verifier](./coh-node/crates/coh-core/src/verify_micro.rs)
- [Lean Formation](./coh-t-stack/Coh/Boundary/Formation.lean)
- [Genesis object](./coh-t-stack/Coh/Boundary/LawOfGenesis.lean)

---

## 10. Next Integration Step: Lean V3 Proof Engineering Loop

### Why Lean V3 First

Clean outcome classes already exist:

| Outcome | Strategy Update |
|---------|-----------------|
| `HelperReductionCompiled` | Increase `HelperDecomposition` weight |
| `MissingLemmaIsolated` | Increase `NamedMissingLemma` / `LemmaDecomposition` |
| `LeanNearMiss` | Small positive if near, otherwise neutral |
| `ForbiddenRejected` | Decrease that strategy sharply |
| Repeated Genesis failure | Reduce wildness/complexity |
| Repeated LeanCompile failure | Shift strategy family |

### First Integration Test

Run two sweeps:

1. **Baseline**: fixed uniform distribution
2. **Adapted**: PhaseLoom-informed distribution after ingesting receipts

Expected improvement:
- Fewer forbidden shortcuts
- More helper/missing-lemma candidates

### Metrics to Track

```rust
struct PhaseLoomMetrics {
    helper_reduction_rate_before: f64,
    helper_reduction_rate_after: f64,
    missing_lemma_rate_before: f64,
    missing_lemma_rate_after: f64,
    forbidden_reject_rate_before: f64,
    forbidden_reject_rate_after: f64,
    strategy_entropy: f64,
    budget_remaining: u128,
    curvature: u128,
    tau: u64,
}
```

### Strategy Entropy (Exploration Floor)

Track Shannon entropy to prevent strategy collapse:

```
H(w) = -Σ w_c * log(w_c)
```

Add policy:
```rust
if strategy_entropy < MIN_ENTROPY {
    inject exploration floor
}
```

### Recommended Files

```text
coh-node/crates/coh-genesis/examples/phaseloom_lean_loop.rs
coh-node/crates/coh-genesis/tests/phaseloom_lean_adaptation_tests.rs
```