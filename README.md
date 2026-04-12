# Coh Validator

**Deterministic AI Verification Kernel & Security Wedge**

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: Proprietary](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)

`ai-safety` `determinism` `rust` `verification-kernel` `integrity-audit`

## Overview

The **Coh Validator** is the deterministic core of the Coh Network's safety stack. It ensures that AI execution traces, receipt chains, and state transitions are cryptographically sound and machine-verifiable.

Built in Rust for maximum memory safety and performance, the validator functions as a "security wedge" between untrusted AI outputs and verifiable on-chain or side-car state.

## Core Features

- **Deterministic Verification**: Zero-tolerance for non-deterministic AI behavior.
- **Receipt Normalization**: Standardizes diverse AI outputs into canonical receipt formats.
- **Cryptographic Hashing**: Uses SHA-256 for integrity auditing of every execution step.
- **Python Bridge**: Seamless integration with Python-based AI workflows via PyO3.
- **Sidecar Service**: High-performance Axum-based API for remote verification.

## Project Structure

- `coh-node/`: The primary Rust workspace.
  - `crates/coh-core/`: Core logic, hashing, and verification kernels.
  - `crates/coh-cli/`: Command-line interface for manual auditing.
  - `crates/coh-python/`: Python bindings for data scientists.
  - `crates/coh-sidecar/`: REST API for network-level verification.

## Development

### Prerequisites

- Rust (latest stable)
- Cargo

### Building

```bash
cd coh-node
cargo build --release
```

### Testing

```bash
cd coh-node
cargo test
```

### Formatting

```bash
cd coh-node
cargo fmt
```

## License

Proprietary - Noetican Labs. All rights reserved.


