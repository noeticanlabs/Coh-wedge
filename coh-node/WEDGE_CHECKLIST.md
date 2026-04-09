# Coh Validator Wedge Stabilization Checklist (v1)

This document tracks the progress of aligning the repository with the locked **Step 1–10 Wedge Specification**.

## Step 1: Freeze the Wedge
- [x] Update README.md to reflect the "Constraint Verifier Engine" identity.
- [x] Rename CLI binary to coh-validator.
- [x] Ensure no "theory-first" framing in public surface.

## Step 2: Freeze the MVP Contract
- [x] Update main.rs to support locked command surface:
  - verify-micro <input.json>
  - verify-chain <input.jsonl>
  - build-slab <input.jsonl> --out <output.json>
  - verify-slab <input.json>
- [x] Implement strict exit code contract (0, 1, 2, 3, 4).
- [x] Support --format text and --format json.

## Step 3: Rust Data Contracts
- [x] Implement four-layer contract in types.rs:
  - Enums: Decision, RejectCode.
  - Wire Structs: MicroReceiptWire, MetricsWire, etc. (String numerics, deny_unknown_fields).
  - Runtime Structs: MicroReceipt, Metrics, etc. (u128 conversion).
  - Prehash Structs: MicroReceiptPrehash, MetricsPrehash (Alphabetized).
- [x] Implement TryFrom<Wire> for runtime parsing.
- [x] Standardize result structs (e.g., VerifyMicroResult).

## Step 4: Make verify-micro Real
- [x] Implement frozen 8-step verification order in verify_micro.rs.
- [x] Enforce policy inequality: v_post + spend <= v_pre + defect.
- [x] Use checked arithmetic everywhere.
- [x] Wire CLI to verify-micro logic.

## Step 5: Fix Canonicalization for Real
- [x] Implement alphabetized prehash serialization in canon.rs.
- [x] Structurally exclude chain_digest_next from prehash.
- [x] Add golden byte-level tests.

## Step 6: Fix Chain Digest Semantics
- [x] Implement non-circular digest rule with DIGEST_DOMAIN_TAG.
- [x] Ensure chain_digest_prev and other critical fields are tied to the digest.

## Step 7: Add Checked Arithmetic Everywhere
- [x] Create math.rs with safe_add, safe_sub, safe_mul.
- [x] Replace all raw arithmetic operators in verifier logic.

## Step 8: Make verify-chain Real
- [x] Implement verify_chain.rs supporting .jsonl input.
- [x] Enforce contiguous step_index, state_hash links, and chain_digest links.
- [x] Report exact failing step index.

## Step 9: Make build-slab Real
- [x] Implement build_slab.rs with deterministic Merkle root (leaves = chain_digest_next).
- [x] Perform checked aggregation of totals.
- [x] Handle exit code 4 for invalid source chains.

## Step 10: Make verify-slab Real
- [x] Implement verify_slab.rs with standalone macro inequality check.
- [x] Enforce range sanity and micro_count validation.
- [x] Ensure no chain replay in v1 standalone mode.

---

## Required Fixture Pack
- [x] examples/micro_valid.json
- [x] examples/micro_invalid_policy.json
- [x] examples/micro_invalid_digest.json
- [x] examples/micro_malformed.json
- [x] examples/chain_valid.jsonl
- [x] examples/chain_invalid_digest.jsonl
- [x] examples/chain_invalid_state_link.jsonl
- [x] examples/chain_invalid_step_index.jsonl
- [x] examples/chain_malformed.jsonl
- [x] examples/slab_valid.json
- [x] examples/slab_invalid_summary.json

## Required Verification
- [x] Digest stability vector test.
- [x] Canonicalization golden test.
- [x] CLI exit-code integration tests.
