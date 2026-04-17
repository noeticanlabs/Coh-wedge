# RFAP V1.0 Compliance Implementation Plan

> **Goal**: Achieve full Rigor-First AI Protocol (RFAP) V1.0 compliance for the Coh Safety Wedge.

---

## Executive Summary

Based on the internal audit report, the Coh Safety Wedge demonstrates strong engineering maturity with a machine-verified formal foundation (Lean 4) and a deterministic verification kernel (Rust). However, to achieve **RFAP V1.0 Compliance**, three categories of "rigor gaps" must be addressed:

1. **Explicit Rejects** (Rust): Remove unsafe `.unwrap()` calls in consensus-critical paths.
2. **API Consistency** (Python): Unify exception behavior across verify/verify_chain APIs.
3. **No-Bluff Proofs** (Lean): Replace placeholder `axiom` declarations with actual proofs.

---

## Compliance Matrix

| Category | Component | Current State | RFAP Target | Priority |
|----------|-----------|---------------|-------------|----------|
| Explicit Rejects | `verify_micro.rs:131` | `.unwrap()` on canon | Explicit `RejectCode` propagation | P0 |
| Explicit Rejects | `verify_chain.rs:48` | Redundant `.unwrap()` | Remove redundant conversion | P0 |
| API Consistency | `coh-python:112` | `CohVerificationError` | Unified result object | P1 |
| API Consistency | `coh-python:137` | Returns `VerifyChainResult` | Align with `verify()` | P1 |
| No-Bluff Proofs | `HashBridge.lean:46` | `axiom digestUpdate_refines_sha256_spec` | Theorem proof | P2 |
| No-Bluff Proofs | `HashBridge.lean:52` | `axiom compute_chain_digest_eq_spec` | Theorem proof | P2 |

---

## Detailed Implementation Steps

### Phase 1: Rust Kernel Hardening (P0)

#### 1.1 Fix `verify_micro.rs` (Line 131)
**Current Code**:
```rust
let canon_bytes = to_canonical_json_bytes(&prehash).unwrap();
```
**Problem**: Violates "Explicit Rejects" rule—canonicalization failure should propagate a `RejectCode`.
**Required Change**: Replace with explicit error handling that returns a `VerifyMicroResult` with `Decision::Reject` and appropriate `RejectCode` (e.g., `RejectCanonProfile` or a new `RejectCanonFailure`).

#### 1.2 Fix `verify_chain.rs` (Line 48)
**Current Code**:
```rust
let r = MicroReceipt::try_from(wire).unwrap();
```
**Problem**: Redundant conversion—`verify_micro` already validated the wire (line 33). Using `unwrap()` here is unsafe.
**Required Change**: Remove the redundant `try_from` call entirely, or refactor to reuse the `MicroReceipt` returned from `verify_micro` if feasible (requires refactoring `verify_micro` to return the receipt on success).

---

### Phase 2: Python Bindings Unification (P1)

#### 2.1 Unify Exception Behavior
**Current State**:
- `verify()` raises `CohVerificationError` on failure.
- `verify_chain_api()` returns a `VerifyChainResult` object (success or failure encoded in the struct).

**Required Change**:
- Option A: Make `verify_chain_api()` raise `CohVerificationError` on failure (aligning with `verify()`).
- Option B: Make `verify()` return a result object (aligning with `verify_chain_api()`).

**Recommendation**: Option A is more idiomatic for Python (`raise on error`) and aligns with the "Explicit Rejects" philosophy. `verify_chain_api()` should raise `CohVerificationError` when `result.decision == Decision::Reject`.

---

### Phase 3: Lean Proof Layer "No-Bluff" (P2)

#### 3.1 Replace Axioms in `HashBridge.lean`

**Current State**:
```lean
axiom digestUpdate_refines_sha256_spec
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev r.canonicalPayload =
      sha256_spec (rustChainDigestInputBytes r.chainDigestPrev
        (receiptProjectionCanonicalJson (receiptProjectionOf r)))

axiom compute_chain_digest_eq_spec
    (r : MicroReceipt)
    (hPayload : PayloadMatchesCanonicalJson r) :
    digestUpdate r.chainDigestPrev r.canonicalPayload =
      sha256_spec ...
```

**Required Change**: These must be proven theorems, not axioms. The proofs should demonstrate that the Rust implementation (`digestUpdate`) refines the mathematical specification (`sha256_spec`).

**Proof Strategy**:
1. Define the relationship between Rust's byte representation and Lean's mathematical objects.
2. Prove that canonicalization produces byte arrays that satisfy the SHA-256 spec preimage requirements.
3. Use the `sha256_spec` equational theory to derive the equality.

> **Note**: This is the highest-effort change. It may require additional lemmas about byte equivalence and canonical form uniqueness.

---

## Testing & Verification

To confirm compliance after implementation:

1. **Rust Tests**: Run `cargo test` — all 7 CLI verification vectors must pass.
2. **Lean Build**: Run `cd coh-t-stack && lake build` — must result in zero `sorry` or `axiom` violations.
3. **Python Tests**: Run `pytest` (if available) or manual verify/verify_chain calls to confirm unified exception behavior.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| `unwrap()` count in consensus paths | 0 |
| Python API divergence | 0 (unified error model) |
| Axiom declarations in `Coh.*` namespace | 0 |
| Lake build status | Green (no warnings) |
| CLI vector pass rate | 7/7 (100%) |

---

## Execution Order

1. **Phase 1 (Rust)**: Critical path—removes unsafe state transitions.
2. **Phase 2 (Python)**: UX consistency—aligns error handling.
3. **Phase 3 (Lean)**: Formal foundation—achieves "no-bluff" proof status.

---