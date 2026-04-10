# ?? Coh Validator

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Protocol: Coh V1](https://img.shields.io/badge/Protocol-Coh_V1-blueviolet.svg)](#)
[![Identity: Frozen Wedge](https://img.shields.io/badge/Identity-Frozen_Wedge-blue.svg)](#)

> **"Coh Validator is a deterministic CLI tool that verifies state transitions, detects tampering in transition chains, and explains invalid actions with explicit reject codes."**

The **Coh Validator** is the reference "Frozen Wedge" implementation for the Coh protocol. It serves as a high-rigor, deterministic constraint verifier engine.

---

## ?? The Safety Kernel (Core Invariant)

The primary job of the validator is to enforce the **Accounting Law of Transitions**. For every micro-receipt, the system ensures that:

V_{post} + spend \le V_{pre} + defect

Failure to satisfy this inequality results in a **RejectPolicyViolation** decision.

---

## ??? Command Surface

### 1. erify-micro <input.json>
Verifies a single transition receipt in isolation.

**Example Accept**:
`	ext
ACCEPT
step_index: 0
object_id: obj_123
chain_digest_next: d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149
`

### 2. erify-chain <input.jsonl>
Verifies a contiguous chain of receipts.

**Example Accept**:
`	ext
ACCEPT
steps_verified: 2
first_step_index: 0
last_step_index: 1
final_chain_digest: 1fa90ecefbd25df4c47848c66e919ca5676b21255173c850cc3110df6ee51114
`

### 3. uild-slab <input.jsonl> --out <output.json>
Aggregates a verified chain into a single high-level **Slab Receipt**.

**Example Success**:
`	ext
SLAB_BUILT
message: Slab built successfully
range_start: 0
range_end: 1
micro_count: 2
merkle_root: 5c6e9d8f5f8f2d29fdc4871d96b3018f7cae37729eaf76f3f944897b104ce650
output: examples/slab_valid.json
`

### 4. erify-slab <input.json>
Verifies a standalone slab-receipt using macro-accounting logic.

**Example Accept**:
`	ext
ACCEPT
range_start: 0
range_end: 1
micro_count: 2
merkle_root: 5c6e9d8f5f8f2d29fdc4871d96b3018f7cae37729eaf76f3f944897b104ce650
`

---

## ??? Technical Specification

### 4-Layer Data Model
1. **Wire Layer**: All numerical fields are encoded as **Decimal Strings**.
2. **Runtime Layer**: Converted to **u128** for exact-integer arithmetic.
3. **Prehash Layer**: Alphabetized canonical view for deterministic hashing.
4. **Result Layer**: Typed decisions (ACCEPT/REJECT/SLAB_BUILT).

### Exit Code Contract
| Code | Label | Description |
| :--- | :--- | :--- |
| **0** | SUCCESS | Verification successful or Slab built. |
| **1** | REJECT | Semantic rejection (Policy violation, Linkage failure). |
| **2** | MALFORMED | Input error (JSON parse error, Missing fields). |
| **3** | ERROR | Internal execution error. |
| **4** | SOURCE | Invalid source chain provided to uild-slab. |

### Reject Code Taxonomy
- RejectSchema: Invalid schema ID or version.
- RejectCanonProfile: Profile hash mismatch.
- RejectChainDigest: Digest linkage or integrity failure.
- RejectStateHashLink: State transition discontinuity.
- RejectNumericParse: Invalid decimal string.
- RejectOverflow: Arithmetic overflow.
- RejectPolicyViolation: Accounting law breach.
- RejectSlabSummary: Slab macro-accounting failure.
- RejectSlabMerkle: Slab Merkle root mismatch.

---

## ?? Getting Started

### Installation
`ash
cargo build --release -p coh-validator
`

### Running Examples
`ash
# Valid chain verification
coh-validator verify-chain examples/chain_valid.jsonl

# Build a slab
coh-validator build-slab examples/chain_valid.jsonl --out examples/slab_new.json
`

---

**Built with rigor by the Antigravity Team.**
