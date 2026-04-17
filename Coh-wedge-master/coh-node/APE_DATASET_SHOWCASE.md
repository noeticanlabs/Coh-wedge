# APE Dataset Showcase

> Complete inventory of valid and invalid workflow chains for APE agent selection

---

## Overview

This dataset provides the complete set of candidate workflows that an APE (Adversarial Proposal Engine) can choose from. Each workflow chain is designed to test different aspects of the Coh verification kernel.

| Category | Location | Count | Purpose |
|----------|----------|-------|---------|
| Valid | `vectors/valid/` | 3 | Baseline correct behavior |
| Adversarial | `vectors/adversarial/` | 6 | Attack vector testing |
| Semi-realistic | `vectors/semi_realistic/` | 2 | Edge case coverage |

---

## Valid Workflows (APPROVED)

These workflows pass all Coh verification checks and result in ACCEPT.

### 1. valid_chain_10.jsonl

```
Location: coh-node/vectors/valid/valid_chain_10.jsonl
Steps: 10
Expected Result: ACCEPT
Description: Standard 10-step valid workflow chain
```

**Sample Step (Step 0):**
```json
{
  "step_index": 0,
  "object_id": "agent.valid.10",
  "state_hash_prev": "0000...0000",
  "state_hash_next": "0000...0001",
  "metrics": { "v_pre": "100", "v_post": "99", "spend": "1", "defect": "2" }
}
```

### 2. valid_chain_100.jsonl

```
Location: coh-node/vectors/valid/valid_chain_100.jsonl
Steps: 100
Expected Result: ACCEPT
Description: Extended 100-step valid workflow chain
```

### 3. valid_chain_1000.jsonl

```
Location: coh-node/vectors/valid/valid_chain_1000.jsonl
Steps: 1000
Expected Result: ACCEPT
Description: Stress test with 1000-step valid workflow
```

---

## Invalid Workflows (REJECTED BY COH)

These workflows are designed to be rejected by the verification kernel at specific points.

### 1. reject_schema.jsonl

```
Location: coh-node/vectors/adversarial/reject_schema.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectSchema
Description: Invalid schema ID or version - testing schema validation
```

**Failure Point:** Step 0 - Schema validation fails before any other checks.

### 2. reject_chain_digest.jsonl

```
Location: coh-node/vectors/adversarial/reject_chain_digest.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectChainDigest
Description: Chain digest continuity broken between steps
```

**Failure Point:** The `chain_digest_next` of one step doesn't match `chain_digest_prev` of next step.

### 3. reject_state_link.jsonl

```
Location: coh-node/vectors/adversarial/reject_state_link.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectStateHashLink
Description: State hash discontinuity - state transition invalid
```

**Failure Point:** The `state_hash_next` of one step doesn't match `state_hash_prev` of next step.

### 4. reject_numeric_parse.jsonl

```
Location: coh-node/vectors/adversarial/reject_numeric_parse.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectNumericParse
Description: Invalid numeric format in metrics field
```

**Failure Point:** Malformed hex strings or invalid digit counts in `v_pre`, `v_post`, `spend`, or `defect`.

### 5. reject_overflow.jsonl

```
Location: coh-node/vectors/adversarial/reject_overflow.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectOverflow
Description: Arithmetic overflow in accounting calculation
```

**Failure Point:** `v_post + spend` exceeds u128::MAX.

### 6. reject_policy_violation.jsonl

```
Location: coh-node/vectors/adversarial/reject_policy_violation.jsonl
Expected Result: REJECT (Exit Code 1)
Reject Reason: RejectPolicyViolation
Description: Accounting law violated - v_post + spend > v_pre + defect + authority
```

**Example (Step 1):**
```json
{
  "step_index": 1,
  "metrics": {
    "v_pre": "100",
    "v_post": "50",
    "spend": "9999999",  // VIOLATION: spend > v_pre + defect
    "defect": "0"
  }
}
```

---

## Semi-Realistic Workflows

### 1. ai_workflow_valid.jsonl

```
Location: coh-node/vectors/semi_realistic/ai_workflow_valid.jsonl
Expected Result: ACCEPT
Description: Realistic AI agent workflow with valid state transitions
```

### 2. ai_workflow_noisy.jsonl

```
Location: coh-node/vectors/semi_realistic/ai_workflow_noisy.jsonl
Expected Result: Varies by step
Description: Workflow with edge-case scenarios mixed in
```

---

## Quick Reference: Demo Commands

### Verify Valid Workflows

```bash
# 10-step valid chain
coh-validator.exe verify-chain coh-node/vectors/valid/valid_chain_10.jsonl

# 100-step valid chain
coh-validator.exe verify-chain coh-node/vectors/valid/valid_chain_100.jsonl

# 1000-step valid chain
coh-validator.exe verify-chain coh-node/vectors/valid/valid_chain_1000.jsonl
```

**Expected Output:** ACCEPT (Exit Code 0)

### Verify Invalid Workflows

```bash
# Schema rejection
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_schema.jsonl
# → REJECT, code: RejectSchema

# Chain digest break
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_chain_digest.jsonl
# → REJECT, code: RejectChainDigest

# State link break
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_state_link.jsonl
# → REJECT, code: RejectStateHashLink

# Numeric parse error
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_numeric_parse.jsonl
# → REJECT, code: RejectNumericParse

# Arithmetic overflow
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_overflow.jsonl
# → REJECT, code: RejectOverflow

# Policy violation
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_policy_violation.jsonl
# → REJECT, code: RejectPolicyViolation
```

---

## APE Agent Selection

When APE selects from this dataset:

```
APE Selection Logic:
├── Random pick from valid/ → Returns valid workflow (ACCEPT)
├── Random pick from adversarial/ → Returns attack vector (REJECT)
└── Semi-realistic → Mixed results based on noise content
```

**Total Candidate Count:** 11 workflows
- Valid: 3 (27%)
- Invalid: 6 (55%)
- Semi-realistic: 2 (18%)

---

## Investor Demo Script

For a 2-minute investor demo:

```bash
# 1. Show valid workflow (ACCEPT)
coh-validator.exe verify-chain coh-node/vectors/valid/valid_chain_10.jsonl

# 2. Show policy violation (REJECT)
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_policy_violation.jsonl

# 3. Show state link break (REJECT)
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_state_link.jsonl

# 4. Show schema rejection (REJECT)
coh-validator.exe verify-chain coh-node/vectors/adversarial/reject_schema.jsonl
```

**Demo Narrative:**
> "APE selects from these 11 workflow candidates. Valid ones pass through. Invalid ones are deterministically blocked - here's proof."

---

## File Structure

```
coh-node/vectors/
├── valid/
│   ├── valid_chain_10.jsonl
│   ├── valid_chain_100.jsonl
│   └── valid_chain_1000.jsonl
├── adversarial/
│   ├── reject_schema.jsonl
│   ├── reject_chain_digest.jsonl
│   ├── reject_state_link.jsonl
│   ├── reject_numeric_parse.jsonl
│   ├── reject_overflow.jsonl
│   └── reject_policy_violation.jsonl
└── semi_realistic/
    ├── ai_workflow_valid.jsonl
    └── ai_workflow_noisy.jsonl
```

---

## See Also

- [SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) - Full system flow
- [bench_v1.json](examples/bench_v1.json) - Performance metrics
- [dominance_v1.json](examples/dominance_v1.json) - Adversarial test results
- [ERROR_REJECT_CONTRACT.md](../plans/ERROR_REJECT_CONTRACT.md) - Reject code taxonomy