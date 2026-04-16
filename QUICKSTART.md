# End-to-End Verification Walkthrough

This guide walks through the complete verification pipeline from raw AI action to verified receipt.

---

## Quick Start (5 minutes)

### 1. Verify a Chain

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-chain coh-node/vectors/valid/valid_chain_10.jsonl
```

**Expected Output**:
```
ACCEPT
steps_verified: 10
first_step_index: 0
last_step_index: 9
```

### 2. Build a Slab (Aggregate)

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  build-slab coh-node/vectors/valid/valid_chain_10.jsonl --out coh-node/slab_output.json
```

### 3. Verify the Slab

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-slab coh-node/slab_output.json
```

---

## Complete Pipeline Diagram

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│   AI System         │     │   verify_micro      │     │   verify_chain       │
│   Raw Receipt       │────▶│   (single receipt)   │────▶│   (sequence)         │
│   (JSON)            │     │                     │     │                     │
│                     │     │ Schema check        │     │ Index continuity    │
│ Example:           │     │ Policy check        │     │ State linkage       │
│ {                  │     │ Digest verify       │     │ Digest linkage      │
│   "step_index": 0, │     │                     │     │                     │
│   "v_pre": "100",  │     └─────────────────────┘     └──────────┬──────────┘
│   "v_post": "95",  │                                        │
│   "spend": "5"     │                                        ▼
│ }                  │                            ┌─────────────────────┐
└─────────────────────┘                            │   build_slab        │
                                                     │   (aggregate)       │
                                                     │                     │
                                                     │ Merkle root         │
                                                     │ Summary compute     │
                                                     └──────────┬──────────┘
                                                                │
                                                                ▼
                                                     ┌─────────────────────┐
                                                     │   verify_slab       │
                                                     │   (macro receipt)   │
                                                     │                     │
                                                     │ Range check         │
                                                     │ Merkle verify       │
                                                     │ Macro policy        │
                                                     └─────────────────────┘
```

---

## Input Formats

### Micro Receipt (JSON)

```json
{
  "schema_id": "coh.receipt.micro.v1",
  "step_index": 0,
  "state_pre": "abc123",
  "state_post": "def456",
  "canon_profile_hash": "sha256:...",
  "chain_digest_prev": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "metrics": {
    "v_pre": "1000",
    "v_post": "950",
    "spend": "30",
    "defect": "5",
    "authority": "10"
  }
}
```

### Chain (JSONL - one JSON per line)

```jsonl
{"step_index": 0, ...}
{"step_index": 1, ...}
{"step_index": 2, ...}
```

### Slab (JSON)

```json
{
  "schema_id": "coh.receipt.slab.v1",
  "first_step": 0,
  "last_step": 9,
  "micro_count": 10,
  "merkle_root": "sha256:...",
  "summary": {
    "total_spend": "300",
    "total_defect": "50"
  }
}
```

---

## Reject Codes (Troubleshooting)

| Code | Meaning | Fix |
|------|---------|-----|
| `RejectSchema` | Invalid schema ID/version | Check schema_id matches expected format |
| `RejectPolicyViolation` | Accounting law violated | Ensure `v_post + spend ≤ v_pre + defect + authority` |
| `RejectStateHashLink` | State continuity broken | Check `state_post` of step N matches `state_pre` of step N+1 |
| `RejectChainDigest` | Chain integrity broken | Verify chain digest linkage |
| `RejectOverflow` | Arithmetic overflow | Use checked math (u128 max) |

---

## Testing with Examples

The project includes test vectors in `coh-node/examples/` and `coh-node/vectors/`:

```bash
# Valid examples
ls coh-node/examples/micro_valid.json
ls coh-node/examples/chain_valid.jsonl
ls coh-node/examples/slab_valid.json
ls coh-node/vectors/valid/

# Invalid examples (should all REJECT)
ls coh-node/examples/micro_invalid_*.json
ls coh-node/examples/chain_invalid_*.jsonl
ls coh-node/examples/slab_invalid_*.json

# AI demo examples
ls coh-node/examples/ai_demo/*.json
```

Run all test vectors:

```bash
cargo test --manifest-path coh-node/Cargo.toml
```

---

## Python Integration

```python
import subprocess
import json

# Verify a receipt via CLI
result = subprocess.run(
    ["coh-validator", "verify-micro", "receipt.json"],
    capture_output=True,
    text=True
)

if "ACCEPT" in result.stdout:
    print("Receipt verified!")
else:
    print(f"Rejected: {result.stdout}")
```

---

## Next Steps

- [ ] Try the [Dashboard](coh-dashboard/README.md) for visual inspection
- [ ] Explore [coh-node/](coh-node/) for more examples and vectors
- [ ] Explore [coh-t-stack/](coh-t-stack/) for formal proofs