# Remaining Clippy Issues Fix Plan

## Current State
- **Original warnings:** 22
- **After first pass:** 8
- **Fixed:** 14 issues

## Remaining Issues (8 warnings)

### 1. `too_many_arguments` (3 functions)
**Files:**
- `coh-node/crates/coh-core/tests/test_valid_chain.rs:78` - 9 args
- `coh-node/crates/coh-core/examples/integrity_demo_gen.rs:96` - 8 args
- `coh-node/crates/coh-core/examples/gen_ai_fixtures.rs:250` - 8 args

**Problem:** Functions have too many parameters (>7)

**Solution:** Create a builder pattern or config struct:

```rust
// Before
fn create_wire_with_metrics(
    index: u64,
    prev_digest: String,
    prev_state: String,
    next_state: String,
    v_pre: &str,
    v_post: &str,
    spend: &str,
    defect: &str,
    step_type: &str,
) -> MicroReceiptWire

// After
struct WireConfig {
    index: u64,
    prev_digest: String,
    prev_state: String,
    next_state: String,
    v_pre: String,
    v_post: String,
    spend: String,
    defect: String,
    step_type: String,
}

fn create_wire_with_metrics(config: WireConfig) -> MicroReceiptWire
fn create_wire_with_metrics(index: u64, state: StateConfig) -> MicroReceiptWire
```

**Steps:**
1. Create new config structs in each file
2. Refactor function signatures
3. Update all callers

---

### 2. `unnecessary_cast` (1 instance)
**File:** `coh-node/crates/coh-core/examples/real_agent_integration.rs:41`

**Problem:**
```rust
let v_pre = current_vault_value as u128;  // u128 -> u128 = unnecessary
```

**Solution:** The cast is actually needed for type inference in this case. Alternative approaches:

**Option A:** Make `current_vault_value` be u128 from the start:
```rust
let current_vault_value: u128 = 1000000;  // explicit type
let v_pre = current_vault_value;
```

**Option B:** Use type annotation on the expression:
```rust
let v_pre = <u128>::from(current_vault_value);
```

**Recommended:** Option A - define `current_vault_value` as `u128` at declaration (line ~15).

---

### 3. `unused_mut` (1 instance)
**File:** `coh-node/crates/coh-core/examples/enterprise_benchmark.rs:485`

**Problem:**
```rust
let mut wire = MicroReceiptWire { ... };
// wire gets mutated below at line 627
```

**Solution:** Use interior mutability or restructure:
```rust
// Option A: Compute the value before constructing
let chain_digest_next = compute_chain_digest(...);
let wire = MicroReceiptWire {
    chain_digest_next,
    ..other_fields
};

// Option B: Use Cell/RefCell (less recommended for this case)
```

**Recommended:** Option A - compute the digest during construction rather than mutating after.

---

### 4. `unused_variables` (3 instances)
**File:** `coh-node/crates/coh-core/examples/enterprise_benchmark.rs`

- Line 1055: `result = search(&ctx)` - function has wrong signature
- Line 1130: `error_count` - unused but calculated
- Line 1246: `conc_stats` - unused but calculated

**Problem:** The `search()` function requires 3 arguments but is called with 1. This appears to be incomplete/broken code.

**Solution:**
1. Fix the `search()` call signature to match the actual function
2. Prefix truly unused values with underscore: `_error_count`

---

### 5. `dead_code` (1 struct)
**File:** `coh-node/crates/coh-core/src/vectors_measurement.rs:38`

**Problem:**
```rust
struct ReflectionViolator;  // never used
```

**Solution:**
- Either use it in tests
- Or remove if truly not needed
- Or add `#[allow(dead_code)]` if intentional for future use

---

## Implementation Priority

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| 1 | unused_variables underscore prefix | Low | Quick win |
| 2 | dead_code attribute | Low | Quick win |
| 3 | unnecessary_cast fix | Low | Quick win |
| 4 | too_many_arguments refactor | Medium | API change |
| 5 | unused_mut restructure | Medium | Logic change |
| 6 | search() function fix | High | Investigation needed |

## Files to Modify
1. `coh-node/crates/coh-core/tests/test_valid_chain.rs`
2. `coh-node/crates/coh-core/examples/integrity_demo_gen.rs`
3. `coh-node/crates/coh-core/examples/gen_ai_fixtures.rs`
4. `coh-node/crates/coh-core/examples/real_agent_integration.rs`
5. `coh-node/crates/coh-core/examples/enterprise_benchmark.rs`
6. `coh-node/crates/coh-core/src/vectors_measurement.rs`