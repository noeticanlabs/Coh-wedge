# Coh Error & Reject Contract

> Unified error contract mapping across all Coh interfaces

---

## Layer 1: CLI Process Exit Codes

Coarse-grained process outcomes for shell scripting:

| Exit Code | Meaning | When Triggered |
|----------|---------|----------------|
| **0** | ACCEPT | All verification passed |
| **1** | REJECT | Any rejection (policy, schema, chain, etc.) |
| **2** | MALFORMED | Input file not found / parse error |
| **3** | ERROR | Internal error (file write, etc.) |
| **4** | SOURCE | `build-slab` source chain failed validation before slab output |

**Design Rationale**: 
- Shell branching only needs coarse outcome (accept/reject)
- Fine-grained semantics live in structured outputs
- Codes `0`-`3` are shared across verifier commands
- Code `4` is reserved for `build-slab` only; it is not a universal verifier exit

---

## Layer 2: Reject Codes (Structured)

Fine-grained rejection reasons in JSON payloads:

### All RejectCode Variants

| Code | Category | Semantic Meaning | Common Triggers |
|------|----------|-----------------|-----------------|
| `RejectSchema` | Input | Schema ID or version mismatch | Wrong `schema_id` or `version` |
| `RejectCanonProfile` | Input | Canonical profile mismatch | Non-zero `canon_profile_hash` |
| `RejectChainDigest` | Linkage | Chain digest link broken | `chain_digest_prev` doesn't match previous `chain_digest_next` |
| `RejectStateHashLink` | Linkage | State hash discontinuity | `state_hash_prev` doesn't match previous `state_hash_next` |
| `RejectNumericParse` | Input | Hex/numeric format error | Invalid hex string, overflow, wrong digit count |
| `RejectOverflow` | Math | Arithmetic overflow | v_post + spend exceeds u128::MAX |
| `RejectPolicyViolation` | Policy | Accounting inequality violated | `v_post + spend > v_pre + defect` (or defect > spend) |
| `RejectSlabSummary` | Slab | Summary totals mismatch | Summed metrics don't match claimed `summary` |
| `RejectSlabMerkle` | Slab | Merkle root mismatch |`n| `RejectIntervalInvalid` | Linkage | Transition interval gap detected | Computed Merkle root doesn't match claimed `merkle_root` |

---

## Layer 3: Interface Mapping

### CLI JSON Output

```json
{
  "decision": "REJECT",
  "code": "RejectPolicyViolation",
  "message": "Macro inequality violated: v_post_last + total_spend (...) exceeds v_pre_first + total_defect (...)",
  "step_index": 2,
  "object_id": "agent.workflow.demo"
}
```

### CLI Text Output

```
REJECT
code: RejectPolicyViolation
message: Macro inequality violated: v_post_last + total_spend (150) exceeds v_pre_first + total_defect (42)
step_index: 2
object_id: agent.workflow.demo
```

### Sidecar API Response

```json
{
  "data": null,
  "error": {
    "code": "E004",
    "message": "Macro inequality violated..."
  }
}
```

**Sidecar Error Codes:**

| Sidecar Code | Maps to RejectCode |
|-------------|-----------------|
| E001 | RejectSchema, RejectCanonProfile, RejectNumericParse |
| E003 | RejectChainDigest, RejectStateHashLink |
| E004 | RejectPolicyViolation, RejectSlabSummary, RejectSlabMerkle |
| E005 | Malformed request (bad JSON) |

### Python API

```python
# verify() returns VerifyResult with fields:
result.decision  # "Accept" | "Reject"
result.code     # RejectCode enum
result.message  # Human-readable string
result.step_index
result.object_id

# Or raises exception on parse error
```

---

## Decision Flow Diagram

```
Input Receipt/Slab
       │
       ▼
┌──────────────┐
│ Parse Input  │ ──► RejectNumericParse (E001) ──► exit(2)
└──────────────┘
       │
       ▼
┌──────────────┐
│ Schema ID   │ ──► RejectSchema ──► exit(1) + JSON reject
│ Version    │ ──► RejectSchema ──��� exit(1) + JSON reject
└──────────────┘
       │
       ▼
┌──────────────┐
│ Canon      │ ──► RejectCanonProfile ──► exit(1) + JSON reject
│ Profile    │
└──────────────┘
       │
       ▼
┌──────────────┐
│ Chain       │ ──► RejectChainDigest ──► exit(1) + JSON reject
│ Linkage     │ ──► RejectStateHashLink ──► exit(1) + JSON reject
└──────────────┘
       │
       ▼
┌──────────────┐
│ Math       │ ──► RejectOverflow ──► exit(1) + JSON reject
│ Checks     │
└──────────────┘
       │
       ▼
┌──────────────┐
│ Policy     │ ──► RejectPolicyViolation ──► exit(1) + JSON reject
│ Check      │
└──────────────┘
       │
       ▼
     ACCEPT (exit 0)
```

`build-slab` follows the same malformed/internal error paths, returns `SLAB_BUILT` with exit `0` on success, and exits `4` when the source chain fails verification before slab emission.

---

## Interface Comparison

| Aspect | CLI Exit | CLI JSON | Sidecar | Python |
|--------|---------|---------|--------|--------|
| **Accept** | 0 | `decision: "Accept"` | HTTP 200 + `"data": {...}` | `.decision == Accept` |
| **Reject** | 1 | `decision: "REJECT"` | HTTP 200 + `"error": {...}` | `.decision == Reject` |
| **Parse Error** | 2 | `"code": "RejectNumericParse"` | HTTP 400 | raises exception |
| **Internal Error**| 3 | N/A | HTTP 500 | raises exception |
| **Build Source Failure** | 4 (`build-slab` only) | `decision: "REJECT"` + source-chain `RejectCode` | N/A | N/A |

---

## Usage Examples

### Shell Scripting
```bash
coh-validator verify-micro input.json --format json
if [ $? -eq 0 ]; then
  echo "Accepted"
else
  echo "Rejected - check output for details"
fi
```

### JSON Processing
```bash
coh-validator verify-chain chain.jsonl --format json | jq -r '.code'
# Output: "RejectPolicyViolation" or null
```

### Sidecar Client
```python
import requests
resp = requests.post("http://localhost:3030/v1/verify-chain", json=receipts)
if resp.json().get("error"):
    print(f"Rejected: {resp.json()['error']['code']}")
```

### Python Direct
```python
from coh_python import verify
result = verify(micro_receipt)
if result.decision == "Reject":
    print(f"Code: {result.code}, Message: {result.message}")
