# Coh Wedge — Formal Foundation

This document specifies the mathematical and logical foundations of the Coh Validator system.

---

## Core Invariant: The Accounting Law of Transitions

For every micro-receipt, the system enforces:

```
v_post + spend <= v_pre + defect
```

Where:
- `v_pre` = unresolved risk/value before the agent step
- `v_post` = unresolved risk/value after the agent step  
- `spend` = operational cost / work consumed in this step
- `defect` = tolerated uncertainty / allowed variance / slack

**Failure** to satisfy this inequality results in a `RejectPolicyViolation` decision.

---

## System Layers

### Layer 1: Wire
- All numerical fields encoded as **Decimal Strings**
- JSON format with strict schema enforcement
- `deny_unknown_fields` prevents extra data

### Layer 2: Runtime
- Converted to `u128` for exact-integer arithmetic
- All arithmetic uses checked operations (`checked_add`, `checked_sub`)
- No floating-point, no overflow possible

### Layer 3: Prehash
- Alphabetized canonical view for deterministic hashing
- Structurally excludes `chain_digest_next` to guarantee non-circularity
- Serialized as JSON bytes for digest computation

### Layer 4: Result
- `Decision::Accept` — verification passed
- `Decision::Reject` — verification failed with explicit `RejectCode`
- `Decision::SlabBuilt` — slab construction succeeded

---

## Cryptographic Design

### Digest Computation

```
chain_digest = SHA256("COH_V1_CHAIN" || "|" || prev_digest_bytes || "|" || canonical_json)
```

- Domain tag `COH_V1_CHAIN` prevents cross-context hash collisions
- Uses raw bytes of previous digest (not hex-encoded)
- Canonical JSON ensures deterministic output

### Merkle Root

```
merkle_inner = SHA256("COH_V1_MERKLE" || "|" || left_bytes || "|" || right_bytes)
```

- Domain tag `COH_V1_MERKLE` separates from chain digests
- Odd leaf count handled by self-duplication
- Empty input returns zero hash

---

## Reject Code Taxonomy

| Code | Condition |
|------|-----------|
| `RejectSchema` | Invalid schema_id or version |
| `RejectCanonProfile` | Canon profile hash mismatch |
| `RejectChainDigest` | Digest linkage or integrity failure |
| `RejectStateHashLink` | State transition discontinuity |
| `RejectNumericParse` | Invalid decimal string format |
| `RejectOverflow` | Arithmetic overflow in checked math |
| `RejectPolicyViolation` | Accounting law inequality violated |
| `RejectSlabSummary` | Slab macro-accounting failure |
| `RejectSlabMerkle` | Slab Merkle root mismatch |

---

## Verification Functions

### verify_micro
1. Parse wire to runtime (hex + numeric validation)
2. Check schema_id and version
3. Verify canon profile hash
4. Verify policy inequality (checked arithmetic)
5. Compute and verify cryptographic digest

### verify_chain
1. For each receipt:
   - Call verify_micro
   - Verify step_index is strictly +1 from previous
   - Verify chain_digest_prev matches previous chain_digest_next
   - Verify state_hash_prev matches previous state_hash_next
2. Return decision with first failing step index

### build_slab
1. Call verify_chain on entire receipt vector
2. Aggregate totals: `total_spend`, `total_defect` (checked arithmetic)
3. Build Merkle tree from chain_digest_next of each receipt
4. Construct SlabReceiptWire with computed merkle_root

### verify_slab (standalone)
1. Parse wire to runtime
2. Verify schema and version
3. Verify range and count consistency
4. Verify macro inequality: `v_post_last + total_spend <= v_pre_first + total_defect`
5. Return Accept/Reject with details

### verify_slab_with_leaves (full verification)
1. Run verify_slab for schema/range/policy checks
2. Extract chain digests from receipts (leaves)
3. Compute Merkle root from leaves
4. Compare computed root against slab's merkle_root
5. Return RejectSlabMerkle if mismatch

---

## Determinism Guarantees

- No floating-point arithmetic
- No randomness / RNG
- No external system calls (time, network, filesystem)
- Canonical JSON ordering ensures identical digest for identical semantic input
- Checked arithmetic prevents overflow-based attacks

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| verify_micro | O(1) | Single JSON parse + SHA256 |
| verify_chain | O(n) | Linear in chain length |
| build_slab | O(n) | Includes chain verification |
| verify_slab | O(1) | Standalone slab check |
| verify_slab_with_leaves | O(n) | Full verification with Merkle |

---

## Copyright

This document is proprietary to **NoeticanLabs (Micheal Ellington)**. All rights reserved.

See [`LICENSE`](LICENSE) for governing terms.



# Lean to Rust Traceability

The core accounting invariant is **formally proved** in Lean 4. This document maps each formal theorem to its Rust enforcement point.

Lean repository: [github.com/noeticanlabs/coh-lean](https://github.com/noeticanlabs/coh-lean)

---

## The IsLawful Predicate

In `Coh/Core/Chain.lean`, the `IsLawful` predicate formalizes the single-step accounting law:

```lean
def IsLawful (r : MicroReceipt) : Prop :=
  r.metrics.v_post + r.metrics.spend ≤ r.metrics.v_pre + r.metrics.defect
```

**Rust enforcement**: Steps 5 of `verify_micro` in `crates/coh-core/src/verify_micro.rs`:

```rust
// Constraint: v_post + spend <= v_pre + defect
let lhs = r.metrics.v_post.safe_add(r.metrics.spend)?;
let rhs = r.metrics.v_pre.safe_add(r.metrics.defect)?;
if lhs > rhs { return Reject(RejectPolicyViolation) }
```

---

## The lawful_composition Theorem

In Lean, the composition theorem proves that if every micro-step in a chain is lawful, the aggregate slab is also lawful:

```lean
theorem lawful_composition (chain : List MicroReceipt) (h : ∀ r ∈ chain, IsLawful r) :
    v_post_last + total_spend ≤ v_pre_first + total_defect
```

**Rust enforcement**: `verify_slab` in `crates/coh-core/src/verify_slab.rs` enforces the aggregate:

```rust
// Macro inequality: v_post_last + total_spend <= v_pre_first + total_defect
let lhs = r.summary.v_post_last.safe_add(r.summary.total_spend)?;
let rhs = r.summary.v_pre_first.safe_add(r.summary.total_defect)?;
if lhs > rhs { return Reject(RejectSlabSummary) }
```

---

## Arithmetic Safety

The Lean proof assumes exact integer arithmetic. In Rust, this is guaranteed by:

- All metrics stored as `u128`
- All arithmetic via `CheckedMath` trait (`safe_add`, `safe_sub`, `safe_mul`)
- No floating-point in any path

Any overflow that the Lean proof did not account for is surfaced as `RejectOverflow` rather than silently wrapping.

---

## Digest Non-Circularity

The Lean model specifies that the digest of a receipt must not include the digest field itself (preventing circular self-reference). In Rust, the prehash view (`MicroReceiptPrehash`) structurally omits `chain_digest_next`:

```rust
pub struct MicroReceiptPrehash {
    pub canon_profile_hash: String,
    pub chain_digest_prev: String,   // previous digest IS included
    // chain_digest_next is NOT a field here
    pub metrics: MetricsPrehash,
    // ...
}
```

---

## Traceability Summary

| Lean Construct | Rust Location | Enforcement Point |
|---|---|---|
| `IsLawful` predicate | `verify_micro.rs` step 5 | Policy inequality check |
| `lawful_composition` | `verify_slab.rs` | Slab macro-inequality |
| Exact integer arithmetic | `math.rs` `CheckedMath` | All arithmetic operations |
| Non-circular digest | `types.rs` `MicroReceiptPrehash` | Structural field exclusion |
| Alphabetized canon | `canon.rs` `to_prehash_view` | JSON byte ordering |
