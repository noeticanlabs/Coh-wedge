# 🌌 Coh Validator Node

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Protocol: Coh V1](https://img.shields.io/badge/Protocol-Coh_V1-blueviolet.svg)](#)

> **The executable witness of the Coh Safety Kernel.**

The **Coh Validator Node** is the reference execution environment for the Coh protocol. It implements the deterministic verifier spine formalized in the [Coh-Lean](https://github.com/noeticanlabs/coh-lean) safety kernel.

This node is designed for **maximum rigor**, ensuring that every transition is receipted, every history is verifiable in O(1), and the entire state-linkage is cryptographically anchored.

---

## 🛠️ Key Features

### 💎 Deterministic Verifier
- **Zero-Float Logic**: All thermodynamic calculations use `QFixed` (i128) fixed-point math.
- **Strict Rejection**: Precise, ordered reject codes that match the formal semantics byte-for-byte.
- **Canonical Hashing**: Implements the `COH_V1` domain-separated hashing protocol.

### ⛓️ Micro & Macro Receipts
- **Micro Receipts**: Step-by-step auditing of carrier transitions and thermodynamic metrics.
- **Slab Receipts**: O(1) compression of historical chains into verifiable macro-summaries.
- **Merkle Commitments**: Transparent history with efficient inclusion challenges.

### 🛡️ Safety Invariants
- **Accounting Law**: $V_{post} + spend \le V_{pre} + defect$
- **Digest Linkage**: Cryptographic chaining of global state and ledger digests.

---

## 🏗️ Architecture

```text
coh-node/
├── crates/
│   ├── coh-core/    # Protocol library (The Safety Kernel)
│   └── coh-cli/     # Demo & scenario runner
├── docs/            # Deep-dive architectural documentation
└── vectors/         # Deterministic test vectors for regression
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust Stable](https://rustup.rs/) (1.94+)

### Build

```bash
cargo build --release
```

### Run the Demo

```bash
cargo run -p coh-cli -- run-demo
```

### Run Tests

```bash
cargo test --workspace
```

---

## 📖 Documentation

Explore the protocol details in the `docs/` folder:

- [00: Purpose & Scope](./docs/00-purpose-and-scope.md)
- [01: Canonical Data Model](./docs/01-canonical-data-model.md)
- [02: Verifier Ordering](./docs/02-verifier-ordering.md)
- [03: Chain & Slab Laws](./docs/03-chain-and-slab-laws.md)
- [07: Lean-to-Rust Traceability](./docs/07-lean-to-rust-traceability.md)

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.

---

**Built with rigor by the Antigravity Team.**
