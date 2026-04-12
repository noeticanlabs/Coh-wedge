# Coh Wedge Refactor Checklist

> Exact file-by-file refactor plan based on audit findings

## Overview

This document provides an exact refactor checklist to address the audit findings. The plan is organized by priority and file.

---

## Priority 1: Tighten Canon Honesty

### Problem
The repo claims to be a "full Coh implementation" but is actually a "deterministic receipt validator wedge".

### Actions

#### `coh-node/README.md` (lines 1-20)
- [x] Change tagline from "deterministic state transition validator" to "Coh wedge kernel - deterministic receipt validator subset"
- [x] Add disclaimer: "This is a subset aligned to Coh v1 contract, not the full canon implementation"

#### `coh-node/crates/coh-core/README.md` (lines 1-20)
- [x] Update overview to clarify this is a "wedge kernel" not "full implementation"
- [x] Add section "Contract Alignment Status" listing what's implemented vs. planned

#### `coh-node/docs/00-purpose-and-scope.md`
- [x] Add clear distinction between "implemented now" vs. "planned canon extensions"

---

## Priority 2: Replace faux-canonical JSON with real canon profile

### Problem
`to_canonical_json_bytes()` uses `serde_json::to_vec()` which is not RFC 8785/JCS-grade.

### File: `coh-node/crates/coh-core/src/canon.rs`

#### Changes (lines 31-33)
- [x] Replace `serde_json::to_vec` with proper JCS canonicalization
- [x] Implement RFC 8785 rules:
  - No unnecessary whitespace
  - Sort keys lexicographically
  - Use escaped unicode (not raw)
  - Encode numbers as integers where possible
- [x] Add `to_canonical_json_bytes_jcs()` function

#### Note
This is a load-bearing issue for cross-language consensus. The change should maintain backward compatibility with existing fixtures by providing a migration path.

---

## Priority 3: Extend receipt and reject schema

### File: `coh-node/crates/coh-core/src/types.rs` (lines 52-64)

#### MicroReceiptWire - Add fields
- [x] Add `step_type: Option<String>` field (for future step categorization)
- [x] Add `signatures: Option<Vec<SignatureWire>>` field (for multi-party signatures)

#### Add new types
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureWire {
    pub signer: String,
    pub signature: String,
    pub timestamp: u64,
}
```

#### Runtime layer updates
- [x] Add corresponding fields to `MicroReceipt` runtime struct
- [x] Update `try_from` conversion logic

### File: `coh-node/crates/coh-core/src/reject.rs`

#### RejectCode enum - Add variant (line 14)
- [x] Add `RejectIntervalInvalid` to enum (for interval validation failures)

---

## Priority 4: Clarify slab verification modes

### File: `coh-node/crates/coh-core/src/verify_slab.rs` (lines 7-11)

#### Current state
The NOTE already exists but is easy to miss.

#### Actions
- [x] Rename function to `verify_slab_envelope()` (more descriptive)
- [x] Add prominent doc comment:
  ```rust
  /// NOTE: This verifies macro-accounting integrity but does NOT verify the Merkle root.
  /// Full Merkle verification requires `verify_slab_with_leaves()`.
  /// - `verify_slab_envelope()` = summary/envelope verification only
  /// - `verify_slab_with_leaves()` = full merkle verification
  ```
- [x] Export both functions with clear distinction in lib.rs

---

## Priority 5: Pull Lean proof layer into deliverable form

### Problem
The `coh-lean/` directory exists but the audit notes it feels "implied but not delivered."

### Actions

#### Option A: Include actual Lean contents (if they exist)
- [x] Verify coh-lean contents are actually populated
- [x] If populated: Add to release artifacts/zips

#### Option B: Clarify delivery status
- [x] Update `coh-node/README.md` to clarify Lean status:
  ```
  ### Formal Proof Layer
  The coh-lean directory contains the Lean4 mechanization of the Coh contract.
  Status: [Include in release / Future deliverable / Reference only]
  ```

#### File: `coh-lean/README.md`
- [x] Ensure clear status indication of the Lean proof

---

## Priority 6: Upgrade dashboard tests

### Problem
Tests are fixture-shape tests, not behavioral UI tests.

### File: `coh-dashboard/src/App.test.jsx`

#### Current tests (fixture-based)
- [x] `can load files`
- [x] `can parse JSONL`
- [x] `fixture fields exist`

#### Add behavioral tests
- [x] Add test: `scenario switching` - verify switching between valid/invalid chains updates UI
- [x] Add test: `broken-chain state rendering` - verify error display for broken chains
- [x] Add test: `slab-fail rendering` - verify slab verification failure display
- [x] Add test: `sidecar fallback` - verify behavior when sidecar is unavailable
- [x] Add test: `proof payload visibility` - verify proof data is visible in UI
- [x] Add test: `selected-step synchronization` - verify selected step updates metrics display

### File: `coh-dashboard/vitest.config.js`
- [x] Ensure testing library (react-testing-library) is available

---

## Summary of File Changes

| File | Priority | Change Type |
|------|----------|-------------|
| `coh-node/README.md` | 1 | Language tightening |
| `coh-node/crates/coh-core/README.md` | 1 | Language tightening |
| `coh-node/docs/00-purpose-and-scope.md` | 1 | Add disclaimer |
| `coh-node/crates/coh-core/src/canon.rs` | 2 | JCS canonicalization |
| `coh-node/crates/coh-core/src/types.rs` | 3 | Schema extension |
| `coh-node/crates/coh-core/src/reject.rs` | 3 | Add RejectIntervalInvalid |
| `coh-node/crates/coh-core/src/verify_slab.rs` | 4 | Rename/clarify functions |
| `coh-node/crates/coh-core/src/lib.rs` | 4 | Export updates |
| `coh-lean/README.md` | 5 | Status clarification |
| `coh-dashboard/src/App.test.jsx` | 6 | Add behavioral tests |

---

## Execution Order

1. **Priority 3** (schema/reject) - Foundation for other changes
2. **Priority 2** (canon) - Load-bearing technical fix
3. **Priority 4** (slab verification) - Documentation clarity
4. **Priority 1** (language tightening) - High-impact credibility fix
5. **Priority 5** (Lean) - Clarify delivery status
6. **Priority 6** (dashboard tests) - Test coverage improvement
