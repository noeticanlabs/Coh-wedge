# Production Audit Runbook

## Overview

This runbook documents the audit procedures, thresholds, and testing requirements for bringing the Coh Safety Wedge system to production grade.

## Audit Checklist

### 1. Core Invariant Verification

| Check | Threshold | Status |
|-------|-----------|--------|
| Accounting law holds | `v_post + spend <= v_pre + defect` for all valid inputs | [ ] |
| Policy violations rejected | Exit code 1 for all policy violations | [ ] |
| Malformed inputs rejected | Exit code 2 for all parse errors | [ ] |
| Schema validation | Proper rejection with RejectSchema code | [ ] |

### 2. Property-Based Test Coverage

| Property | Test Count | Minimum |
|----------|------------|---------|
| Accounting law | Minimum 100 variations | [ ] |
| Boundary cases | Edge at equality, edge +1 | [ ] |
| Overflow resistance | Max values tested | [ ] |
| Determinism | Same input = same output (3x) | [ ] |
| Vacuous rejection | All-zero receipts | [ ] |

### 3. Differential Testing

| Check | Requirement |
|-------|--------------|
| V1 vs V3 consistency | Both implementations agree on Acceptance/Rejection |
| Same decision codes | Policy violation uses same codes |
| Boundary alignment | Both accept at exact boundary |

### 4. Adversarial Coverage

| Vector Type | Count | Location |
|-------------|-------|----------|
| Valid chains | 1000+ | `vectors/valid/` |
| Policy violations | 50+ | `vectors/adversarial/reject_policy_violation.jsonl` |
| Edge cases | 50+ | `vectors/adversarial/reject_edge_cases.jsonl` |
| Schema failures | 20+ | `vectors/adversarial/reject_schema.jsonl` |
| Numeric parse failures | 20+ | `vectors/adversarial/reject_numeric_parse.jsonl` |

### 5. Performance Benchmarks

| Metric | Threshold | Measurement |
|-------|-----------|-------------|
| Single receipt verify | < 1ms | P50 latency |
| Chain verify (100 steps) | < 100ms | P95 latency |
| Slab build (1000 receipts) | < 500ms | P95 latency |
| Memory usage | < 50MB | Peak RSS |

### 6. CI/CD Requirements

| Check | Requirement |
|-------|--------------|
| All tests pass | `cargo test -p coh-core` returns 0 |
| Property tests | Include proptest/quickcheck runs |
| Adversarial vectors | All adversarial vectors must REJECT |
| Valid vectors | All valid vectors must ACCEPT |
| Format check | `cargo fmt --check` passes |
| Lint | `cargo clippy` passes with -D warnings |

### 7. Security Requirements

| Check | Requirement |
|-------|--------------|
| No panic on invalid input | All malformed inputs handled gracefully |
| No DoS via overflow | Large integers handled safely |
| Hash collision resistance | SHA-256 only |
| Signature required | Unsigned receipts rejected |

## Running Production Audits

### Daily Audit

```bash
# Run all unit tests
cargo test -p coh-core

# Run property-based tests
cargo test -p coh-core --test test_property

# Run differential tests
cargo test -p coh-core --test test_differential

# Verify adversarial vectors
cargo run -p coh-validator -- verify-chain vectors/adversarial/reject_policy_violation.jsonl
# Expected: exit code 1 (REJECT)
```

### Weekly Fuzzing Audit

```bash
# Run with property-based testing (1000 iterations)
cargo test -p coh-core --test test_property -- --iterations 1000

# Check edge case coverage
cargo test -p coh-core --test test_property test_boundary

# Performance regression check
cargo run -p coh-core --release --example enterprise_benchmark
```

### Monthly Security Audit

```bash
# Full adversarial suite
for f in vectors/adversarial/*.jsonl; do
  cargo run -p coh-validator -- verify-chain "$f"
  if [ $? -eq 0 ]; then
    echo "FAIL: $f should reject but accepted"
    exit 1
  fi
done

# Full valid suite
for f in vectors/valid/*.jsonl; do
  cargo run -p coh-validator -- verify-chain "$f"
  if [ $? -ne 0 ]; then
    echo "FAIL: $f should accept but rejected"
    exit 1
  fi
done
```

## Threshold Definitions

### Accept/Reject Decision Codes

| Code | Meaning | When |
|------|---------|------|
| 0 | ACCEPT | All checks pass |
| 1 | REJECT | Policy violation |
| 2 | REJECT | Malformed input |
| 3 | REJECT | State/link error |
| 4 | SOURCE_ERROR | Build-specific error |

### Reject Codes (Detailed)

| Code | Description | Trigger |
|------|-------------|----------|
| RejectSchema | Invalid schema_id/version | Schema mismatch |
| RejectPolicyViolation | v_post + spend > v_pre + defect | Accounting law broken |
| RejectMissingSignature | No signatures | Empty signature list |
| RejectMissingObjectId | Empty object_id | Empty object ID |
| VacuousZeroReceipt | All metrics zero | No economic activity |
| RejectCanonProfile | Invalid canon profile | Profile mismatch |
| RejectNumericParse | Invalid hex/numeric | Parse failure |
| RejectStateLink | State chain break | Digest mismatch |

## Test Vector Naming Conventions

Valid vectors:
- `valid_chain_N.jsonl` - Valid chain with N steps
- `valid_gccp_*.jsonl` - Valid GCCP transitions
- `semi_realistic/*.jsonl` - Realistic but synthetic

Adversarial vectors:
- `reject_*.jsonl` - Should be rejected
- `reject_policy_violation.jsonl` - Accounting law violations
- `reject_schema.jsonl` - Schema failures
- `reject_edge_cases.jsonl` - Boundary and edge cases

## Troubleshooting

### Common Issues

1. **Property test failure**: Review invariant, adjust strategy
2. **Differential mismatch**: Compare V1 vs V3 implementations
3. **Performance regression**: Check benchmark output
4. **Adversarial vector accepted**: Review rejection logic

## Release Criteria

Before production release:

- [ ] All unit tests pass
- [ ] All property-based tests pass (1000+ iterations)
- [ ] All differential tests pass
- [ ] All adversarial vectors rejected
- [ ] All valid vectors accepted
- [ ] Performance thresholds met
- [ ] No clippy warnings
- [ ] Code formatted correctly
- [ ] Benchmarks captured

---

**Last Updated**: 2024
**Version**: 1.0.0