# APE Trajectory Verification Architecture

## The Core Insight

APE doesn't just verify individual receipts—it verifies **admissible computation trajectories**.

A trajectory:
```
τ = (S₀ → S₁ → S₂ → …)
```

The verifier answers: "Is this trajectory admissible under all constraints?"

## Unified Framework

All attack types become "bad trajectories":

| Attack Type | Trajectory Failure |
|------------|-------------------|
| Mutation | Local trajectory perturbation breaks continuity |
| Recombination | Trajectory splice creates invalid sequence |
| Temporal Drift | Long-horizon divergence |
| Non-Termination | Infinite trajectory (no terminal state) |
| Livelock | Cyclic trajectory (never improves) |
| Resource Exhaustion | Trajectory diverges in cost space |

## Required Invariants

### 1. Local Validity
Every state must be valid:
```
∀t, Sₜ ∈ V
```

### 2. Progress or Termination
Must either terminate or make progress:
```
∃t, terminal(Sₜ) OR P(Sₜ₊₁) < P(Sₜ)
```

### 3. Bounded Cost
Finite resource consumption:
```
∑ₜ cost(Sₜ) < ∞
```

## Practical Implementation

### Trajectory Constraints to Add:
1. **Max chain length** - bounded trajectory depth
2. **Max step count** - prevents infinite iteration
3. **No repeated states** - prevents cycles
4. **Progress metric must change** - ensures forward motion
5. **Cumulative cost bound** - resource budget

### verify_chain upgrades:
- Add step count validation
- Add repeated state detection
- Add progress metric checking
- Add cumulative cost tracking

## Investor Pitch

> "We don't just verify individual AI actions. We verify that sequences of actions form admissible trajectories—preventing infinite loops, drift, and unsafe long-horizon behavior."

## Current State

APE 15-strategy implementation provides foundation:
- Generates adversarial trajectories
- Tests local receipt validity
- Next: add trajectory-level constraints

## Files to Modify

1. `coh-node/crates/coh-core/src/verify_chain.rs` - add trajectory checks
2. `coh-node/crates/coh-core/src/types.rs` - add trajectory metrics
3. `ape/src/strategies/runtime.rs` - enhance trajectory generation
