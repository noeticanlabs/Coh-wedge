# Kernel Architecture

The Coh Safety Wedge is built on a modular kernel architecture that implements the Genesis-Coherence Boundary Theory. The core system operates through the coordination of three primary kernels, unified by the `GmiGovernor`.

## 1. NPE Kernel (Noetic Proposal Engine)

**Location:** `crates/coh-npe/src/kernel.rs`

The NPE Kernel is the "forward generator" responsible for proposing possibilities that satisfy the Law of Genesis.

- **Role:** Propose actions and transitions.
- **State:** Tracks generative configuration and sequence numbering.
- **Governing State:** Manages the accumulation of generative "disorder."
- **Budget:** Enforces bounds on how much disorder the generator can accumulate before a coherence step is required.
- **Key Law:** $M(g') + C(p) \le M(g) + D(p)$ (Proposals must be strictly admissible).

## 2. PhaseLoom Kernel

**Location:** `crates/coh-phaseloom/src/kernel.rs`

The PhaseLoom Kernel manages the boundary strategy ecology. It is an exploration/exploitation mechanism that observes outcomes and adjusts strategy weights.

- **Role:** Observes success/failure rates of different proposal strategies.
- **State:** Tracks weights and tension across different strategies (e.g., Lean theorems, Rust patches).
- **Circuit Breaker:** Will halt the system if "tension" exceeds the configured threshold.
- **Sampling:** Can provide the next strategy to try based on Boltzmann exploration or pure exploitation.

## 3. RV Kernel (Receipt Verification)

**Location:** `crates/coh-core/src/rv_kernel.rs`

The RV Kernel is the "backward verifier" that enforces the Law of Coherence. It sits strictly behind the generative boundary.

- **Role:** Verifies receipts deterministically.
- **State:** Tracks the current safe valuation.
- **Governing State:** Tracks systemic authority and structural boundaries.
- **Budget:** Ensures that cumulative spending does not breach the allowable variance.
- **Key Law:** $V_{post} + Spend \le V_{pre} + Defect + Authority$.

## 4. GmiGovernor

**Location:** `crates/coh-genesis/src/lib.rs`

The `GmiGovernor` unifies the three kernels into a single formation boundary. It acts as the traffic controller, routing proposals from the NPE, tracking them in PhaseLoom, and finally committing them via the RV Kernel.

```rust
pub struct GmiGovernor {
    pub npe: NpeKernel,
    pub rv: RvKernel,
    pub phaseloom: PhaseLoomKernel,
    pub env: EnvironmentalEnvelope,
    pub system: Option<SystemProperties>,
}
```

The Governor enforces the full **Formation Cycle**:
1. NPE generates a candidate action.
2. PhaseLoom tracks the strategy tension.
3. The action is proposed.
4. If successful, an execution receipt is generated.
5. RV Kernel verifies the receipt.
6. The Governor records the outcome back into PhaseLoom and updates the valuation.
