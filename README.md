# Coh Safety Wedge

**Deterministic AI Verification Kernel & Formal T-Stack Ledger**

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Lean: v4.16.0](https://img.shields.io/badge/lean-v4.16.0-blue.svg)](https://leanprover.github.io/)
[![License: Proprietary](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)

`ai-safety` `determinism` `rust` `lean4` `verification-kernel` `integrity-audit`

## Overview

The **Coh Safety Wedge** is the high-integrity core of the Coh Network. It provides a dual-layer security guarantee:
1. **Rust Verification Kernel**: A high-performance, deterministic engine for auditing AI receipt chains and state transitions.
2. **Lean T-Stack Ledger**: A machine-verified formal foundation proving the categorical and physical invariants of the safety contract.

## Project Structure

- **`coh-node/`**: The production Rust workspace.
  - `crates/coh-core/`: Core verification logic (JCS, SHA-256, Accounting Law).
  - `crates/coh-cli/`: `coh-validator` CLI for manual and automated auditing.
  - `crates/coh-python/`: High-level bindings for AI workflow integration.
  - `crates/coh-sidecar/`: Axum-based REST API for remote verification.
- **`coh-t-stack/`**: The Formal T-Stack Ledger (Lean 4).
  - `Coh/Ledger/`: Verified theorems (T1: Strict Coh ? Category).
- **`coh-dashboard/`**: The Integrity Inspector (React/Vite).
  - Visual timeline and audit inspector for AI receipt chains.

## Formal Foundations

The system is anchored by the **T-Stack Federated Ledger**. Every foundational claim is machine-verified using Lean 4 to ensure total mathematical soundness. See [FORMAL_FOUNDATION.md](FORMAL_FOUNDATION.md) for the complete theorem mapping.

## Development

### Prerequisites
- Rust stable
- Lean 4 (Elan)
- Node.js 20+

### Building the Kernel
```bash
cd coh-node
cargo build --release
```

### Building the Ledger
```bash
cd coh-t-stack
lake build
```

### Running the Dashboard
```bash
cd coh-dashboard
npm install
npm run dev
```

## License
Proprietary - Noetican Labs. All rights reserved.
