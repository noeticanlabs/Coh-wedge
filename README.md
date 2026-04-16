# Coh Safety Wedge

**Deterministic verification kernel for AI receipt chains, with a React audit console and a Lean 4 formal layer.**

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Lean: v4.16.0](https://img.shields.io/badge/lean-v4.16.0-blue.svg)](https://leanprover.github.io/)
[![License: Proprietary](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)

`ai-safety` `determinism` `rust` `lean4` `verification-kernel` `integrity-audit`

## Overview

**Coh Safety Wedge** is the trust boundary between untrusted agent output and committed application state.

Instead of accepting an LLM's narration of what happened, Coh requires machine-checkable receipts and rejects workflows that break canonical structure, chain continuity, or the accounting law. The repository combines three layers:

1. **Rust verification kernel** in `coh-node/` for deterministic receipt validation.
2. **React audit console** in `coh-dashboard/` for replaying and inspecting valid and invalid chains.
3. **Lean 4 formal layer** in `coh-t-stack/` for theorem work and formalization artifacts around the protocol model.

## Why It Exists

AI systems fail silently when they:

- report state transitions that never happened,
- skip required intermediate steps,
- produce impossible accounting updates,
- or continue execution after integrity has already broken.

Coh turns those hidden failures into explicit `ACCEPT` / `REJECT` decisions with stable machine-readable surfaces.

## What Coh Verifies

The current repository centers on a frozen V1 verification surface:

- **Micro receipts**: a single step and its local accounting constraints.
- **Receipt chains**: ordered JSONL workflows with state-hash and chain-digest linkage.
- **Slab receipts**: aggregated summaries with macro-accounting and Merkle integrity.
- **Operator visibility**: dashboard and sidecar surfaces for inspecting outcomes.

## Core Invariant

The core accounting law enforced by the current V1 documentation surface is:

```text
v_post + spend <= v_pre + defect
```

| Term | Meaning |
|---|---|
| `v_pre` | Value or unresolved risk before the step |
| `v_post` | Value or unresolved risk after the step |
| `spend` | Resources consumed by the step |
| `defect` | Explicitly tolerated slack or variance |

If the inequality fails, the workflow is rejected before state should be committed.

## Repository Map

| Path | Purpose | Start here |
|---|---|---|
| `coh-node/` | Rust workspace containing the verifier, CLI, Python bindings, sidecar, fixtures, and protocol docs | [`coh-node/README.md`](coh-node/README.md) |
| `coh-dashboard/` | React/Vite dashboard for replaying demo chains and optionally calling the live sidecar | [`coh-dashboard/README.md`](coh-dashboard/README.md) |
| `coh-t-stack/` | Lean 4 formalization workspace and related theorem artifacts | [`FORMAL_FOUNDATION.md`](FORMAL_FOUNDATION.md) |
| `ape/` | Adversarial fixture and experimental support area | [`ape/fixtures/README.md`](ape/fixtures/README.md) |
| `plans/` | Design notes, audits, roadmap material, and implementation plans | [`plans/DOCUMENTATION_AND_SPEC_PLAN.md`](plans/DOCUMENTATION_AND_SPEC_PLAN.md) |

## Quick Start

New to the repo? Start with [`QUICKSTART.md`](QUICKSTART.md), then use the flows below.

### 1. Build the CLI verifier

```bash
cargo build --manifest-path coh-node/Cargo.toml -p coh-validator --release
```

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-chain coh-node/vectors/valid/valid_chain_10.jsonl
```

### 3. Build and verify a slab

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  build-slab coh-node/vectors/valid/valid_chain_10.jsonl --out coh-node/slab.json

cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-slab coh-node/slab.json
```

### 4. Start the sidecar API

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-sidecar --release
```

Default routes are `/health`, `/v1/verify-micro`, `/v1/verify-chain`, and `/v1/execute-verified`.

### 5. Launch the dashboard

```bash
cd coh-dashboard
npm install
npm run dev
```

The dashboard works in fixture mode by default and can optionally call the live sidecar at `http://127.0.0.1:3030`.

## Operational Flow

```text
Untrusted agent output
        |
        v
Adapter / receipt emission
        |
        v
Coh verifier (micro -> chain -> slab)
        |
        +--> ACCEPT -> commit / publish / archive
        |
        +--> REJECT -> halt / alert / inspect
```

## Documentation Guide

| Topic | Document |
|---|---|
| End-to-end walkthrough | [`QUICKSTART.md`](QUICKSTART.md) |
| Rust workspace overview | [`coh-node/README.md`](coh-node/README.md) |
| Dashboard usage | [`coh-dashboard/README.md`](coh-dashboard/README.md) |
| Lean 4 formal layer | [`coh-t-stack/`](coh-t-stack/) |

## Development Prerequisites

- Rust stable
- Node.js 20+
- Lean 4 via Elan/Lake for formal work

## Common Development Commands

### Rust workspace

```bash
cargo test --manifest-path coh-node/Cargo.toml
```

### Dashboard

```bash
cd coh-dashboard
npm run test:run
npm run build
```

### Lean workspace

```bash
cd coh-t-stack
lake build
```

## Docker

For containerized verifier usage:

```bash
docker build -f coh-node/Dockerfile -t coh-validator .
docker-compose up --profile interactive
```

## License

Proprietary software owned by Noetican Labs. See [`LICENSE`](LICENSE) for governing terms.
