# Adversarial Fixtures

Surgical failure cases for testing APE integration with Coh Wedge.

## Fixture Files

### Single Receipt Fixtures (JSON)
| File | Target Invariant | Expected Rejection |
|------|-----------------|-------------------|
| [`valid_micro.json`](valid_micro.json) | None (valid baseline) | Accept |
| [`invalid_accounting.json`](invalid_accounting.json) | Accounting law (v_post = v_pre - spend) | Reject (accounting violation) |
| [`overflow.json`](overflow.json) | Numeric overflow | Reject (overflow) |
| [`invalid_schema.json`](invalid_schema.json) | Schema validation | Reject (invalid schema) |

### Chain Fixtures (JSONL)
| File | Target Invariant | Expected Rejection |
|------|-----------------|-------------------|
| [`valid_chain.jsonl`](valid_chain.jsonl) | Valid chain | Accept |
| [`invalid_state_link.jsonl`](invalid_state_link.jsonl) | State continuity | Reject (state link broken) |
| [`invalid_chain_digest.jsonl`](invalid_chain_digest.jsonl) | Digest continuity | Reject (digest mismatch) |

## Format

Each fixture is a MicroReceiptWire or chain of MicroReceiptWire JSON that targets a specific verification failure mode.

### Loading Fixtures

```rust
use ape::fixtures::{load_micro, load_chain};

// Load single receipt
let receipt = load_micro("valid_micro").expect("Failed to load");
let result = verify_micro(receipt);

// Load chain
let chain = load_chain("valid_chain").expect("Failed to load");
let result = verify_chain(chain);
```

## Test Suite

Run the adversarial tests:
```bash
cd ape
cargo test --test adversarial
```

All 6 tests pass:
- `test_invalid_accounting_reject` - Verifies accounting law violations are caught
- `test_invalid_schema_reject` - Verifies wrong schema ID is caught
- `test_overflow_reject` - Verifies numeric overflow is caught
- `test_invalid_state_link_reject` - Verifies state continuity violations are caught
- `test_invalid_chain_digest_reject` - Verifies chain digest violations are caught
- `test_valid_chain_integration` - Verifies valid chains load correctly

## Notes

- The `valid_chain.jsonl` fixture works with the external `verify_chain` example: `cargo run -p coh-core --example verify_chain ../ape/fixtures/valid_chain.jsonl`
- Some internal API differences between test and example modes require the integration test to verify fixture loading rather than full verification