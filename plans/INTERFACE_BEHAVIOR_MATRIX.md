# Interface Behavior Matrix

> Exact behavior mapping across all Coh interfaces per scenario

| Scenario | Verifier Decision | CLI Exit | CLI JSON `decision` | CLI JSON `code` | Sidecar HTTP | Sidecar Body `error.code` | Python `.decision` | Python `.code` |
|----------|---------------|---------|-------------|-----------------|---------------|------------|-------------------|--------------|------------------|
| Valid accept | ACCEPT | 0 | `"Accept"` | `null` | 200 | `null` | `"Accept"` | `None` |
| Reject: schema mismatch | REJECT | 1 | `"REJECT"` | `"RejectSchema"` | 200 | `"E001"` | `"Reject"` | `"RejectSchema"` |
| Reject: version mismatch | REJECT | 1 | `"REJECT"` | `"RejectSchema"` | 200 | `"E001"` | `"Reject"` | `"RejectSchema"` |
| Reject: canon profile | REJECT | 1 | `"REJECT"` | `"RejectCanonProfile"` | 200 | `"E001"` | `"Reject"` | `"RejectCanonProfile"` |
| Reject: policy violation | REJECT | 1 | `"REJECT"` | `"RejectPolicyViolation"` | 200 | `"E004"` | `"Reject"` | `"RejectPolicyViolation"` |
| Reject: chain digest | REJECT | 1 | `"REJECT"` | `"RejectChainDigest"` | 200 | `"E003"` | `"Reject"` | `"RejectChainDigest"` |
| Reject: state hash link | REJECT | 1 | `"REJECT"` | `"RejectStateHashLink"` | 200 | `"E003"` | `"Reject"` | `"RejectStateHashLink"` |
| Reject: slab summary | REJECT | 1 | `"REJECT"` | `"RejectSlabSummary"` | 200 | `"E004"` | `"Reject"` | `"RejectSlabSummary"` |
| Reject: slab merkle | REJECT | 1 | `"REJECT"` | `"RejectSlabMerkle"` | 200 | `"E004"` | `"Reject"` | `"RejectSlabMerkle"` |
| Malformed: JSON parse | REJECT | 2 | `"REJECT"` | `"RejectNumericParse"` | 200 | `"E001"` | `"Reject"` | `"RejectNumericParse"` |
| Malformed: bad hex | REJECT | 2 | `"REJECT"` | `"RejectNumericParse"` | 200 | `"E001"` | `"Reject"` | `"RejectNumericParse"` |
| Malformed: missing field | REJECT | 2 | `"REJECT"` | `"RejectNumericParse"` | 200 | `"E001"` | `"Reject"` | `"RejectNumericParse"` |
| Error: file not found | — | 2 | — | — | 400 | — | raises exception | — |
| Error: file write fail | — | 3 | — | — | 500 | — | raises exception | — |
| Error: internal panic | — | 3 | — | — | 500 | — | raises exception | — |
| Reject: interval invalid | REJECT | 1 | "REJECT" | "RejectIntervalInvalid" | 200 | "E003" | "Reject" | "RejectIntervalInvalid" |`n| `build-slab`: source chain invalid | REJECT | 4 | `"REJECT"` | `"RejectChainDigest"` / `"RejectStateHashLink"` / source index discontinuity path | N/A | N/A | N/A | N/A |

---

## Key Design Principles

1. **CLI Exit = Coarse**: verifier commands use 0-3; `build-slab` additionally reserves 4 for source-chain failure
2. **JSON/Sidecar/Python = Fine-grained**: Include full `RejectCode` for debugging
3. **HTTP 200 for REJECT**: Sidecar returns 200 with error body (not HTTP error) for verifier rejection decisions
4. **HTTP 400/500 for transport errors**: Only for non-verification errors (malformed request, internal error)

---

## Usage Examples

### Shell Scripting
```bash
coh-validator verify-micro input.json --format json
if [ $? -eq 0 ]; then
  echo "Verified!"
fi
```

### JSON Processing  
```bash
coh-validator verify-chain chain.jsonl --format json | jq -r '.code // "ACCEPT"'
```

### Sidecar Client
```python
import requests
resp = requests.post("http://localhost:3030/v1/verify-chain", json=receipts)
body = resp.json()
if body.get("error"):
    print(f"Rejected: {body['error']['code']}")  # E001, E003, E004, etc.
```

### Python Direct
```python
from coh_python import verify
result = verify(micro_receipt)
if result.decision == "Reject":
    print(f"Rejected: {result.code}")  # RejectPolicyViolation, etc.
```

## Sidecar Error Code Mapping

| Sidecar Code | Maps From RejectCode | Category |
|-------------|-----------------|---------|
| E001 | RejectSchema, RejectCanonProfile, RejectNumericParse | Input/validation |
| E003 | RejectChainDigest, RejectStateHashLink | Chain linkage |
| E004 | RejectPolicyViolation, RejectSlabSummary, RejectSlabMerkle | Verification logic |
| E005 | — | Malformed request (bad JSON) |

Reference: `plans/ERROR_REJECT_CONTRACT.md` for full RejectCode taxonomy.
