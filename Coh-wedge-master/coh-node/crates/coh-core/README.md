# Coherent Validator Core

A deterministic state transition validator with cryptographic tamper detection for blockchain execution verification.

## Overview

This crate provides the core verification engine for state transition receipts. It validates:

- **Micro-receipts**: Individual state transition records
- **Chains**: Linked sequences of receipts with continuity enforcement  
- **Slabs**: Compressed macro receipts with Merkle root integrity

## Features

- **Deterministic verification**: Same input always produces same output
- **Cryptographic tamper detection**: SHA256 digest verification
- **Arithmetic safety**: Checked math prevents overflow attacks
- **Policy enforcement**: v_post + spend <= v_pre + defect
- **Continuity enforcement**: Step order, state linkage, digest linkage
- **Merkle integrity**: Slab verification via Merkle root

## Installation

```toml
[dependencies]
coh-core = "0.1.0"
```

## Quick Start

```rust
use coh_core::{verify_micro, verify_chain, build_slab, verify_slab};
use coh_core::types::{MicroReceiptWire, Decision};
use serde_json::from_str;

// Verify a single receipt
let json = r#"{"schema_id":"coh.receipt.micro.v1",...}"#;
let receipt: MicroReceiptWire = from_str(json).unwrap();
let result = verify_micro(receipt);

match result.decision {
    Decision::Accept => println!("Verified!"),
    Decision::Reject => println!("Rejected: {:?}", result.code),
}
```

## CLI Usage

```bash
# Verify a single receipt
coh-validator verify-micro examples/micro_valid.json

# Verify a chain
coh-validator verify-chain examples/chain_valid.jsonl

# Build a slab from chain
coh-validator build-slab examples/chain_valid.jsonl --out slab.json

# Verify a slab
coh-validator verify-slab examples/slab_valid.json
```

## API Reference

### verify_micro

Validates a single micro-receipt:

```rust
pub fn verify_micro(wire: MicroReceiptWire) -> VerifyMicroResult
```

Checks:
- Schema ID and version
- Canon profile hash
- Policy arithmetic (no overflow, inequality holds)
- Cryptographic digest

### verify_chain

Validates a sequence of receipts:

```rust
pub fn verify_chain(receipts: Vec<MicroReceiptWire>) -> VerifyChainResult
```

Checks:
- Each receipt individually
- Step index continuity (strictly +1)
- State hash linkage
- Chain digest linkage

### build_slab

Constructs a macro receipt from a chain:

```rust
pub fn build_slab(receipts: Vec<MicroReceiptWire>) -> BuildSlabResult
```

Produces:
- Range (first_step, last_step)
- Micro count
- Merkle root
- Aggregate accounting

### verify_slab

Validates a standalone slab:

```rust
pub fn verify_slab(wire: SlabReceiptWire) -> VerifySlabResult
```

Checks:
- Range and count consistency
- Merkle root integrity
- Macro policy inequality
- Summary arithmetic

## Lean 4 Traceability

This crate's verification logic is connected to formal proofs in the Lean T-Stack:

| Lean Theorem | Invariant | Rust Function | Verification Method |
|--------------|-----------|---------------|---------------------|
| `T1_Category.Closure` | Valid state transitions | `verify_micro::check_state_link` | Unit test + property test |
| `T2_OplaxBridge.admissibility` | Policy bounds respected | `verify_micro::check_policy` | Fuzz test |
| `T3_MacroSlab.telescope` | Chain → slab aggregation | `build_slab::aggregate` | Golden test |
| `T4_Visibility.anomaly_detected` | Invalid inputs → reject | `reject.rs` exhaustive match | Coverage 100% |
| `AccountingLaw.budget` | v_post + spend ≤ v_pre + defect + authority | `verify_micro::check_accounting` | Property-based test |

See [FORMAL_FOUNDATION.md](../../FORMAL_FOUNDATION.md) for the complete theorem mapping.

## Types

### Decision

```rust
pub enum Decision {
    Accept,
    Reject,
    SlabBuilt,
}
```

### RejectCode

```rust
pub enum RejectCode {
    RejectSchema,
    RejectCanonProfile,
    RejectChainDigest,
    RejectStateHashLink,
    RejectNumericParse,
    RejectOverflow,
    RejectPolicyViolation,
    RejectSlabSummary,
    RejectSlabMerkle,
}
```

## Examples

Run the included examples:

```bash
# Verify a single receipt from JSON
cargo run --release --package coh-core --example verify_single

# Verify a chain from JSONL
cargo run --release --package coh-core --example verify_chain

# Run performance benchmarks
cargo run --release --package coh-core --example benchmark

# Run stress tests (10K, 100K, streaming)
cargo run --release --package coh-core --example stress_test
```

See `examples/` directory for JSON format examples:
- `micro_valid.json` - Valid micro receipt
- `chain_valid.jsonl` - Valid chain (JSONL)
- `slab_valid.json` - Valid slab

## Testing

```bash
cargo test
```

Run specific test suites:
```bash
cargo test --test test_verify_micro
cargo test --test test_verify_chain
cargo test --test test_verify_slab
```

## Performance (Release Build)

**Benchmark Results (single-threaded, release build, optimized):**

| Operation | Throughput | Latency (avg) |
|-----------|------------|---------------|
| verify-micro | ~104,000 ops/sec | ~8.7 µs |
| verify-chain(10) | ~78,000 ops/sec | ~13 µs/step |
| verify-chain(100) | ~84,000 ops/sec | ~12 µs/step |
| verify-chain(1K) | ~78,000 ops/sec | ~13 µs/step |
| verify-chain(10K) | ~71,000 ops/sec | ~14 µs/step |
| build-slab(100) | N/A | ~21 µs/receipt |
| build-slab(1K) | N/A | ~24 µs/receipt |

**Latency Distribution (micro verification):**
- p50 (median): 7.7 µs
- p95: 11.1 µs
- p99: 28.8 µs
- min: 7.5 µs
- max: 50.8 µs

**CPU Breakdown (estimated):**
- JSON parsing: ~35-40%
- String allocations: ~15-20%
- SHA256 hashing: ~25-30%
- Arithmetic/logic: ~10-15%

**Key insight:** Release build is ~13x faster than debug build. JSON parsing is the bottleneck, NOT hashing.

**Scaling:** Linear performance verified up to 10K+ receipts with no degradation.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Coherent Validator                       │
├─────────────────────────────────────────────────────────────┤
│  Micro Layer         │  Chain Layer       │  Slab Layer      │
│  ─────────────────────────────────────────────────────────  │
│  verify_micro()  →  verify_chain()   →  build_slab()        │
│         ↓                ↓                  ↓               │
│  Single receipt    Linked sequence      Aggregate           │
│  Schema check      Step continuity      Merkle root          │
│  Policy check      State linkage        Summary check        │
│  Digest verify    Digest linkage       Macro policy         │
└─────────────────────────────────────────────────────────────┘
```

## License

Proprietary software owned by **NoeticanLabs (Micheal Ellington)**. All rights reserved. No commercial use, redistribution, or branding use is permitted without prior written permission. See the repository [`LICENSE`](LICENSE) for governing terms.