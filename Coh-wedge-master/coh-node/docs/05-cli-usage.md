# CLI Usage

## Binary

```
coh-validator <COMMAND> [OPTIONS]
```

Build with:
```bash
cd coh-node
cargo build --release -p coh-validator
# Binary: target/release/coh-validator
```

---

## Commands

### verify-micro

Verify a single micro-receipt.

```bash
coh-validator verify-micro <input.json>
coh-validator verify-micro examples/micro_valid.json
coh-validator verify-micro examples/micro_invalid_policy.json --format json
```

Checks: schema, version, object_id, canon profile, policy inequality, chain digest.

---

### verify-chain

Verify a contiguous JSONL chain (one receipt per line).

```bash
coh-validator verify-chain <input.jsonl>
coh-validator verify-chain examples/chain_valid.jsonl
```

Checks: all micro checks, plus step_index continuity, state_hash linkage, chain_digest linkage.

Reports the exact failing step index on rejection.

---

### build-slab

Aggregate a verified chain into a Slab Receipt.

```bash
coh-validator build-slab <input.jsonl> --out <output.json>
coh-validator build-slab examples/chain_valid.jsonl --out /tmp/slab.json
```

Checks: full chain verification before aggregation. Writes slab JSON to `--out`.

---

### verify-slab

Verify a standalone Slab Receipt.

```bash
coh-validator verify-slab <input.json>
coh-validator verify-slab examples/slab_valid.json
```

Checks: schema, version, range/count sanity, macro accounting inequality.

---

## Output Formats

All commands support `--format`:

| Flag | Output |
|---|---|
| `--format text` | Human-readable (default) |
| `--format json` | Machine-readable JSON result struct |

JSON output is suitable for pipeline integration and log aggregation.

---

## Exit Code Contract

For `verify-micro`, `verify-chain`, and `verify-slab`, the CLI uses a shared four-code contract.

| Code | Label | Meaning |
|---|---|---|
| 0 | ACCEPT | Verification passed |
| 1 | REJECT | Semantic rejection (policy violation, schema mismatch, chain break, etc.) |
| 2 | MALFORMED | Input error (file not found, JSON parse failure, invalid hex) |
| 3 | ERROR | Internal execution error (file write failure, etc.) |

`build-slab` uses the same malformed/error handling, returns `0` on successful slab creation, and additionally reserves exit code `4` for source-chain verification failures discovered before slab emission.

| Code | Label | Meaning |
|---|---|---|
| 4 | SOURCE | `build-slab` rejected the input chain as an invalid source for slab construction |

**Note:** Shell scripts should branch on exit code first. For detailed rejection reasons (`RejectCode`), use `--format json` and inspect the `code` field. See `plans/ERROR_REJECT_CONTRACT.md` for the full error taxonomy.

Automation tools should branch on exit code, not parse stdout.

---

## Pipeline Example

```bash
# Verify a chain; on success, build and verify a slab
coh-validator verify-chain agent_run.jsonl \
  && coh-validator build-slab agent_run.jsonl --out run_slab.json \
  && coh-validator verify-slab run_slab.json
```
