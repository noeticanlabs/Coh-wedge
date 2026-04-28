# Coh Safety Wedge

**Deterministic verification kernel for AI receipt chains, with a React audit console and a Lean 4 formal layer.**

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Lean: v4.16.0](https://img.shields.io/badge/lean-v4.16.0-blue.svg)](https://leanprover.github.io/)

`ai-safety` `determinism` `rust` `lean4` `chaos-coherence` `formation-boundary`

## Overview

**Coh Safety Wedge** formalizes **Chaos–Coherence Boundary Theory**: 
- **Chaos** is forward admissible generation (proposing possibilities).
- **Coherence** is backward admissible justification (verifying actuality).
- **Formation** is their intersection—the stricter V2 profile where AI proposals are cryptographically linked to executable commitments.

## Core Invariants

### 1. Law of Chaos (Forward Generation)
```text
M(g') + C(p) <= M(g) + D(p)
```
Enforced by the **Noetic Proposal Engine (NPE)** to ensure generative discipline.

### 2. Law of Coherence (Backward Verification)
```text
v_post + spend <= v_pre + defect + authority
```
Enforced by the **Verifier Kernel** to ensure deterministic safety.

## Repository Map

| Path | Purpose | Start here |
|---|---|---|
| `coh-node/` | Rust workspace containing the Verifier and NPE | [`coh-node/README.md`](coh-node/README.md) |
| `coh-dashboard/` | React/Vite dashboard for inspecting formation boundaries | [`coh-dashboard/README.md`](coh-dashboard/README.md) |
| `coh-t-stack/` | Lean 4 formalization of the Active Boundary Theorem | [`FORMAL_FOUNDATION.md`](FORMAL_FOUNDATION.md) |
| `plans/` | Audits, roadmap, and Chaos-Coherence implementation plans | [`plans/VERIFIER_GAP_ANALYSIS.md`](plans/VERIFIER_GAP_ANALYSIS.md) |

## Quick Start

### 1. Run the NPE (Chaos Admissibility)
```bash
# Verify a proposal satisfies the Law of Chaos
cargo test -p coh-npe
```

### 2. Verify a Chain (Coherence)
```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-chain coh-node/vectors/valid/valid_chain_10.jsonl
```

## Next Steps

- [ ] Inspect the [Formation Boundary](coh-dashboard/README.md) in the dashboard.
- [ ] Explore [coh-npe](coh-node/crates/coh-npe/) for forward generation logic.
- [ ] Explore [coh-t-stack/](coh-t-stack/) for formal proofs.

## License

Proprietary software owned by Noetican Labs. See [`LICENSE`](LICENSE) for governing terms.
