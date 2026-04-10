# Coh Validator`n`n**Deterministic AI Verification Kernel & Security Wedge**`n`n`ai-safety` `determinism` `rust` `verification-kernel` `integrity-audit`

[![CI](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml/badge.svg)](https://github.com/noeticanlabs/Coh-wedge/actions/workflows/ci.yml)
[![Rust: stable](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: Proprietary](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)

> **"Stops corrupted AI workflows in 16ms with zero false positives."**

**Rust Protocol: Coh V1 | Identity: Frozen Wedge**

The Coh Validator is a deterministic CLI tool that verifies state transitions, detects tampering in transition chains, and explains invalid actions with explicit reject codes. It is the reference "Frozen Wedge" implementation for the Coh protocol — a high-rigor constraint verifier engine bridging the formal semantics of [coh-lean](https://github.com/noeticanlabs/coh-lean) with real-world AI agent execution.

---

## The Safety Kernel (Core Invariant)

The primary job of the validator is to enforce the Accounting Law of Transitions. For every micro-receipt, the system ensures that:

```
V_post + spend <= V_pre + defect
```

Where:
- **V_pre**: Pre-transition potential (metrics)
- **V_post**: Post-transition potential (metrics)
- **spend**: Consumed potential
- **defect**: Injected potential (usually zero in standard transitions)

Failure to satisfy this inequality results in an immediate `RejectPolicyViolation` decision.

---

## Command Reference

The validator exposes exactly four commands, designed for use in automated validation pipelines.

### 1. verify-micro <input.json>

Verifies a single transition receipt in isolation.

- Checks: Schema, Version, Canon Profile, Policy Inequality, and Digest Integrity
- Input: A single JSON receipt file

### 2. verify-chain <input.jsonl>

Verifies a contiguous chain of receipts.

- Checks: All micro-checks, plus state-linkage (state_hash_next_i = state_hash_prev_i+1) and digest-linkage (chain_digest_next_i = chain_digest_prev_i+1)
- Input: A JSONL file where each line is a receipt

### 3. build-slab <input.jsonl> --out <output.json>

Aggregates a verified chain into a single high-level Slab Receipt.

- Checks: Fully verifies the input chain before aggregation
- Output: A standalone slab JSON summarizing the ranges and total aggregation

### 4. verify-slab <input.json>

Verifies a standalone slab-receipt using macro-accounting logic.

- Checks: Range sanity, micro-count validation, and macro-inequality

---

## Technical Specification

### 4-Layer Data Model

To eliminate floating-point ambiguity and non-determinism, the validator uses a strict 4-layer architecture:

1. **Wire Layer**: All numerical fields are encoded as Decimal Strings.
2. **Runtime Layer**: Converted to u128 for exact-integer arithmetic.
3. **Prehash Layer**: Alphabetized canonical view for deterministic hashing.
4. **Result Layer**: Typed decisions (ACCEPT/REJECT) with explicit RejectCode.

### Non-Circular Digest Logic

The validator implements a strict non-circular digest rule. The `chain_digest_next` is computed as:

```
SHA256("COH_V1_CHAIN" || "|" || chain_digest_prev || "|" || canonical_json(prehash_view))
```

The prehash view structurally excludes the `chain_digest_next` field itself, ensuring the digest is a true anchor of the content it receipts.

### Exit Code Contract

Automation tools can rely on the following exit codes:

| Code | Label | Description |
|------|-------|-------------|
| 0 | ACCEPT | Verification successful |
| 1 | REJECT | Semantic rejection (Policy violation, Digest mismatch) |
| 2 | MALFORMED | Input error (JSON parse error, Invalid HEX, Missing fields) |
| 3 | ERROR | Internal execution error |
| 4 | SOURCE | Invalid source chain provided to build-slab |

---

## Licensing

This repository is proprietary software owned by **NoeticanLabs (Micheal Ellington)**. No commercial use, redistribution, hosting, or derivative commercial deployment is permitted without prior written permission. The project name, product identity, and related branding are reserved trademarks/service identifiers of NoeticanLabs.

See [`LICENSE`](LICENSE) for governing terms.

---

## Quick Start (Copy-Paste Ready)

```bash
# Clone and build
git clone https://github.com/noeticanlabs/Coh-wedge.git
cd Coh-wedge/coh-node
cargo build --release -p coh-validator

# Run the 60-second cinematic demo (hallucination breach + circuit break)
cargo run --example showcase -p coh-core --release

# Verify a chain of agent steps
coh-validator verify-chain examples/chain_valid.jsonl
```

---

## Formal Verification

The accounting law is **formally proved** in Lean 4:

> [`github.com/noeticanlabs/coh-lean`](https://github.com/noeticanlabs/coh-lean)

The `IsLawful` predicate in `Coh/Core/Chain.lean` is the mathematical specification that this Rust implementation faithfully enforces. See [`FORMAL_FOUNDATION.md`](FORMAL_FOUNDATION.md) for the full traceability map.

---


## Getting Started

### Installation

```bash
cd coh-node
cargo build --release -p coh-validator
```

### Running the Demo

```bash
# See the hallucination breach + circuit breaker in action (60 seconds)
cargo run --example showcase -p coh-core --release
```

### Running Examples

The `coh-node/examples/` directory contains standard test vectors:

```bash
# Valid micro-receipt
coh-validator verify-micro examples/micro_valid.json

# Invalid policy (policy violation)
coh-validator verify-micro examples/micro_invalid_policy.json
```

### Integration Templates

See [`coh-node/examples/integrations/`](coh-node/examples/integrations/) for copy-paste templates:
- **Generic agent loop** — works with any LLM provider
- **OpenAI function calling** — wraps function-call responses with safety gating

---

## Development

- Tests: `cargo test -p coh-core` — digest stability + fixture oracle
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --check`

---

**Built with rigor by the Antigravity Team.**
