# Test Vectors

All fixture files live in `coh-node/examples/`. They are used by the CLI integration tests in `crates/coh-cli/tests/` and the fixture oracle in `crates/coh-core/tests/`.

---

## Micro-Receipt Fixtures

| File | Expected Result | Notes |
|---|---|---|
| `micro_valid.json` | ACCEPT (exit 0) | All checks pass |
| `micro_invalid_policy.json` | REJECT (exit 1) | `v_post + spend > v_pre + defect` |
| `micro_invalid_digest.json` | REJECT (exit 1) | `chain_digest_next` is wrong |
| `micro_malformed.json` | MALFORMED (exit 2) | Invalid JSON or missing required field |

---

## Chain Fixtures

| File | Expected Result | Notes |
|---|---|---|
| `chain_valid.jsonl` | ACCEPT (exit 0) | Valid 3-step chain |
| `chain_invalid_digest.jsonl` | REJECT (exit 1) | Digest broken at step 2 |
| `chain_invalid_state_link.jsonl` | REJECT (exit 1) | `state_hash_prev` mismatch at step 2 |
| `chain_invalid_step_index.jsonl` | REJECT (exit 1) | Non-contiguous step index |
| `chain_malformed.jsonl` | MALFORMED (exit 2) | Malformed JSON on one line |

---

## Slab Fixtures

| File | Expected Result | Notes |
|---|---|---|
| `slab_valid.json` | ACCEPT (exit 0) | Valid standalone slab |
| `slab_invalid_summary.json` | REJECT (exit 1) | Macro inequality violated |

---

## Digest Stability Test

`crates/coh-core/tests/test_fixtures.rs` contains `test_fixture_oracle_sweep`, which:
1. Loads every fixture file
2. Runs the appropriate verifier
3. Asserts the expected exit code matches the expected result

This test pins the digest computation. If the hashing logic, canonicalization order, or domain tag ever changes, this test will fail — providing a regression boundary for protocol stability.

---

## Adding New Fixtures

New fixtures must:
1. Be valid JSON or JSONL
2. Have a corresponding expected result documented above
3. Be added to the fixture oracle sweep in `test_fixtures.rs`

To generate test vectors programmatically, see `examples/gen_ai_fixtures.rs`.
