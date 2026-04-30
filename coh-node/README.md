# Coh Validator

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Protocol: Coh V1](https://img.shields.io/badge/Protocol-Coh_V1-blueviolet.svg)](#)
[![Function: Deterministic Safety Kernel](https://img.shields.io/badge/Function-Deterministic_Safety_Kernel-blue.svg)](#)

The **Coh Validator** is the Rust workspace that implements the repository's deterministic verification surface for AI workflow receipts.

It is designed to sit between an agent and any durable state boundary: emit receipts, verify them, and only then allow commit or publication.

## V1 Scope

The current frozen verification surface is intentionally narrow:

| Command | Purpose | Input | Output |
|---|---|---|---|
| `verify-micro` | Verify one micro-receipt | JSON | `ACCEPT` or `REJECT` |
| `verify-chain` | Verify an ordered receipt chain | JSONL | `ACCEPT` or first failing step |
| `build-slab` | Aggregate a valid chain into a slab receipt | JSONL | Slab JSON file |
| `verify-slab` | Verify a standalone slab receipt | JSON | `ACCEPT` or `REJECT` |

Out of scope for this V1 surface: networking protocols, distributed consensus, streaming ingestion, and multi-tenant orchestration.

## Workspace Layout

| Path | Role |
|---|---|
| `crates/coh-core/` | Core verification logic and data types, including `RvKernel` |
| `crates/coh-cli/` | `coh-validator` command-line interface |
| `crates/coh-python/` | Python bindings for verifier integration |
| `crates/coh-sidecar/` | Axum-based HTTP service exposing verification routes |
| `crates/coh-genesis/` | Unified `GmiGovernor` and law of genesis closure |
| `crates/coh-npe/` | `NpeKernel` for Noetic Proposal Engine logic |
| `crates/coh-phaseloom/` | `PhaseLoomKernel` for boundary strategy ecology |
| `crates/coh-gccp/` | Governed Compute Control Plane logic |
| `crates/coh-time/` | Time and probability engines |
| `crates/coh-fuzz/` | Fuzzing tools and adversarial vectors |
| `docs/` | Purpose, data model, ordering, laws, Merkle flow, CLI, and test-vector docs |
| `vectors/` | Valid and adversarial chain fixtures used for testing and demos |
| `examples/` | Demo assets and integration-oriented sample material |

## Core Invariant

The current workspace documentation centers on this accounting law:

```text
v_post + spend <= v_pre + defect
```

| Field | Meaning |
|---|---|
| `v_pre` | Value or unresolved risk before the step |
| `v_post` | Value or unresolved risk after the step |
| `spend` | Resources consumed by the step |
| `defect` | Allowed slack or tolerated variance |

If the inequality fails, the verifier returns a rejection rather than allowing downstream state to treat the step as valid.

## Quick Start

From the `coh-node/` directory:

```bash
cargo build --release -p coh-validator
cargo test
```

### Verify a valid chain

```bash
cargo run --release -p coh-validator -- \
  verify-chain vectors/valid/valid_chain_10.jsonl
```

### Inspect an adversarial rejection

```bash
cargo run --release -p coh-validator -- \
  verify-chain vectors/adversarial/reject_policy_violation.jsonl --format json
```

### Build and verify a slab from a valid chain

```bash
cargo run --release -p coh-validator -- \
  build-slab vectors/valid/valid_chain_10.jsonl --out slab.json

cargo run --release -p coh-validator -- \
  verify-slab slab.json
```

## CLI Output Contract

All CLI surfaces support `--format text` and `--format json`.

The documented exit-code contract for the current CLI surface is:

| Exit code | Meaning |
|---|---|
| `0` | Verification passed or slab built successfully |
| `1` | Semantic rejection |
| `2` | Malformed input or parse/load failure |
| `3` | Internal execution error |
| `4` | `build-slab` source-chain failure detected before slab emission |

Use `--format json` if an integration needs the structured reject code and message rather than text output.

## Enterprise Benchmark

The workspace includes an enterprise-grade benchmark suite for investor-ready performance metrics:

```bash
cargo run --release -p coh-core --example enterprise_benchmark
```

### What it measures

| Metric | Description |
|--------|-------------|
| Hardware spec | CPU, RAM, OS, Rust version |
| Chain scaling | Throughput vs chain length (1, 10, 100, 1000) |
| Workflow datasets | Financial, Agent, Ops workflows |
| Confusion matrix | False accept/reject rates |
| Concurrency | Multi-threaded stress testing |
| Sidecar overhead | HTTP vs in-process latency |

### Key results (Ryzen 9 7950X / Reference Hardware)

- **Throughput**: ~175k verifications/sec (single-threaded)
- **Concurrency**: >300k ops/sec (multi-threaded)
- **False Accept Rate**: 0% (invalid receipts rejected)
- **False Reject Rate**: 0% (valid receipts accepted)
- **Latency p99**: ~5.7µs/verification under load

## Sidecar API

Start the HTTP sidecar from `coh-node/`:

```bash
cargo run --release -p coh-sidecar
```

By default the server listens on `127.0.0.1:3030` and exposes:

| Route | Purpose |
|---|---|
| `GET /health` | Health probe |
| `POST /v1/verify-micro` | Verify one receipt |
| `POST /v1/verify-chain` | Verify a chain payload |
| `POST /v1/execute-verified` | Example gate that only returns payload on successful receipt verification |

See [`../plans/SIDECAR_API.md`](../plans/SIDECAR_API.md) for the route contract currently documented in the repository.

## Integration Surfaces

- **Rust library**: use `coh-core` directly inside an agent adapter or service boundary.
- **Python**: see `crates/coh-python/` and [`../plans/PYTHON_BINDINGS.md`](../plans/PYTHON_BINDINGS.md).
- **HTTP sidecar**: run `coh-sidecar` for local or service-based verification.
- **Templates**: see [`examples/integrations/README.md`](examples/integrations/README.md) for adapter-oriented examples.

## Documentation Map

| Topic | Document |
|---|---|
| Purpose and frozen V1 scope | [`docs/00-purpose-and-scope.md`](docs/00-purpose-and-scope.md) |
| Canonical data model | [`docs/01-canonical-data-model.md`](docs/01-canonical-data-model.md) |
| Verifier ordering | [`docs/02-verifier-ordering.md`](docs/02-verifier-ordering.md) |
| Chain and slab laws | [`docs/03-chain-and-slab-laws.md`](docs/03-chain-and-slab-laws.md) |
| Merkle challenge flow | [`docs/04-merkle-challenge-flow.md`](docs/04-merkle-challenge-flow.md) |
| CLI usage | [`docs/05-cli-usage.md`](docs/05-cli-usage.md) |
| Test vectors | [`docs/06-test-vectors.md`](docs/06-test-vectors.md) |
| Kernel Architecture (NPE, PhaseLoom, RV) | [`docs/07-kernel-architecture.md`](docs/07-kernel-architecture.md) |
| Dashboard Integration | [`docs/08-dashboard-integration.md`](docs/08-dashboard-integration.md) |
| Case study | [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md) |

## Determinism Notes

The workspace is organized around deterministic verification rather than probabilistic scoring:

- same input should produce the same decision,
- chain linkage is explicit and machine-checkable,
- invalid arithmetic and malformed numeric strings are rejected,
- and operator tooling can inspect the exact failure surface.

## Licensing

This repository is proprietary software owned by **NoeticanLabs (Micheal Ellington)**. No commercial use, redistribution, hosting, or derivative commercial deployment is permitted without prior written permission. See the repository [`LICENSE`](../LICENSE) for governing terms.
