# GCCP Integration Plan

> **Goal**: Update docs, tests, demos, and CI to use GCCP (Governed Compute Control Plane)

---

## Summary

The GCCP spec ([plans/GCCP_V1_GAP_ANALYSIS.md](plans/GCCP_V1_GAP_ANALYSIS.md)) defines a compute-specific control plane built on top of coh-core. This plan covers integration into:

1. **Documentation** - Reference GCCP in existing docs
2. **Tests** - Add GCCP-specific test vectors
3. **Demos** - Add GCCP demo scenarios  
4. **CI** - Run GCCP benchmarks in pipeline

---

## Phase 1: Documentation Updates

### 1.1 SYSTEM_ARCHITECTURE.md

Update to mention GCCP as the compute specialization layer.

```markdown
### GCCP Integration (v1)

The Coh system provides:
- Generic receipt-verifiable state transitions
- The accounting law: v_post + spend ≤ v_pre + defect + authority

GCCP (Governed Compute Control Plane) specializes Coh for compute:
- Thermal/power/queue pressure functionals
- Compute-specific guard conditions
- Hardware-class state morphisms

See: [plans/GCCP_V1_GAP_ANALYSIS.md](../plans/GCCP_V1_GAP_ANALYSIS.md)
```

**Files to modify:**
- `coh-node/SYSTEM_ARCHITECTURE.md`

### 1.2 README Updates

Add GCCP section to main README.

**Files to modify:**
- `coh-node/README.md`

---

## Phase 2: Test Updates

### 2.1 Add GCCP Test Vectors

Per GCCP spec, need 6 new vectors:
1. `valid_gccp_thermal.csv` - Valid with temp cap
2. `reject_gccp_temp_cap.csv` - Reject on thermal violation
3. `valid_gccp_power.csv` - Valid with power limit
4. `reject_gccp_power_breach.csv` - Reject on power cap
5. `valid_gccp_priority.csv` - Valid queue priority
6. `reject_gccp_throttle.csv` - Reject on throttling

**Locations:**
- `coh-node/vectors/gccp/valid_*.jsonl`
- `coh-node/vectors/gccp/reject_*.jsonl`

### 2.2 Add Unit Tests

Add tests for new GCCP types:
- `coh-node/crates/coh-core/src/gccp/state_tests.rs`
- `coh-node/crates/coh-core/src/gccp/pressure_tests.rs`

### 2.3 Update test_valid_chain.rs

Add GCCP case variants.

**Files to modify:**
- `coh-node/crates/coh-core/tests/test_valid_chain.rs`

---

## Phase 3: Demo Updates

### 3.1 Add GCCP Demo Script

Create demo that showcases GCCP-specific behavior.

**New file:**
- `coh-node/examples/gccp_demo.rs`

### 3.2 Update enterprise_benchmark.rs

Add GCCP configuration option.

**Files to modify:**
- `coh-node/crates/coh-core/examples/enterprise_benchmark.rs`

### 3.3 Add Demo Vectors

Add semi-realistic GCCP workflow vectors.

**Files:**
- `coh-node/vectors/semi_realistic/gccp_ai_workflow.jsonl` (new)

---

## Phase 4: CI Updates

### 4.1 GitHub Actions

Add GCCP to test matrix:

```yaml
jobs:
  test:
    strategy:
      matrix:
        include:
          - mode: "coh"
            vectors: "valid/*,semi_realistic/*"
          - mode: "gccp"
            vectors: "gccp/*"
```

**Files to modify:**
- `.github/workflows/test.yml`

### 4.2 Benchmark CI

Add GCCP benchmarks to nightly runs:

```yaml
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run GCCP benchmarks
        run: cargo bench --features gccp
```

---

## Implementation Order

```
1. Documentation (1h)
   └── Update SYSTEM_ARCHITECTURE.md
   
2. Test Vectors (4h)
   ├── Create 6 GCCP vectors
   └── Add unit tests for state.rs
   
3. Demo (3h)
   ├── Create gccp_demo.rs
   └── Update enterprise_benchmark
   
4. CI (2h)
   ├── Update test.yml
   └── Add GCCP benchmarks
```

---

## Files Summary

### New Files
- `coh-node/vectors/gccp/valid_gccp_thermal.jsonl`
- `coh-node/vectors/gccp/valid_gccp_power.jsonl`
- `coh-node/vectors/gccp/valid_gccp_priority.jsonl`
- `coh-node/vectors/gccp/reject_gccp_temp_cap.jsonl`
- `coh-node/vectors/gccp/reject_gccp_power_breach.jsonl`
- `coh-node/vectors/gccp/reject_gccp_throttle.jsonl`
- `coh-node/crates/coh-core/src/gccp/state_tests.rs` (if module exists)
- `coh-node/examples/gccp_demo.rs`

### Modified Files
- `coh-node/SYSTEM_ARCHITECTURE.md`
- `coh-node/README.md`
- `coh-node/crates/coh-core/tests/test_valid_chain.rs`
- `.github/workflows/test.yml`

---

## Dependencies

GCCP implementation requires:
- `coh-node/crates/coh-core/src/gccp/state.rs` - Must exist (per gap analysis, marked as IMPLEMENTED)
- `coh-node/crates/coh-core/src/gccp/pressure.rs` - May need creation

**Check:** First verify the gccp module exists before proceeding.