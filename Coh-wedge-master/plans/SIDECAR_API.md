# Coh Sidecar API

> HTTP API contract for remote verification

## Base URL

```
http://localhost:3030
```

## Routes

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/verify-micro` | Verify single micro-receipt |
| POST | `/v1/verify-chain` | Verify receipt chain |
| POST | `/v1/execute-verified` | Execute verified action |

---

## POST /v1/verify-micro

Verify a single micro-receipt.

### Request

```json
{
  "schema_id": "coh.receipt.micro.v1",
  "version": "1.0.0",
  "object_id": "agent.workflow.demo",
  "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
  "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "step_index": 0,
  "state_hash_prev": "1111111111111111111111111111111111111111111111111111111111111111",
  "state_hash_next": "2222222222222222222222222222222222222222222222222222222222222222",
  "chain_digest_prev": "0000000000000000000000000000000000000000000000000000000000000000",
  "chain_digest_next": "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c",
  "metrics": {
    "v_pre": "100",
    "v_post": "88",
    "spend": "12",
    "defect": "0"
  }
}
```

### Success Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Accept",
  "data": null,
  "error": null
}
```

### Rejection Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Reject",
  "data": null,
  "error": {
    "code": "E001",
    "message": "Invalid schema_id: wrong.schema (Expected: coh.receipt.micro.v1)",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

## POST /v1/verify-chain

Verify a contiguous receipt chain.

### Request

```json
{
  "receipts": [
    { /* MicroReceiptWire */ },
    { /* MicroReceiptWire */ }
  ]
}
```

### Success Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Accept",
  "data": {
    "break_index": 0,
    "message": ""
  },
  "error": null
}
```

### Rejection Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Reject",
  "data": {
    "break_index": 2,
    "message": "Chain digest link broken at step 2: expected link to abc123... but found def456..."
  },
  "error": {
    "code": "E003",
    "message": "Chain digest link broken at step 2: expected link to abc123... but found def456...",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

## POST /v1/execute-verified

Execute an action after verification.

### Request

```json
{
  "action": {
    "type": "db_commit",
    "payload": {
      "key": "value"
    }
  },
  "receipts": [
    { /* MicroReceiptWire */ }
  ]
}
```

### Success Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Accept",
  "data": {
    "action": {
      "type": "db_commit",
      "executed": true
    }
  },
  "error": null
}
```

### Rejection Response (HTTP 200)

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "coh_version": "0.1.0",
  "status": "Reject",
  "data": null,
  "error": {
    "code": "E003",
    "message": "State hash link broken at step 1: expected state abc... but found def...",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

## Error Codes

| Code | Maps From RejectCode | Category | Description |
|------|-------------------|----------|-------------|
| E001 | RejectSchema, RejectCanonProfile, RejectNumericParse | Input | Schema/version mismatch, invalid input format |
| E003 | RejectChainDigest, RejectStateHashLink | Linkage | Chain digest or state hash discontinuity |
| E004 | RejectPolicyViolation, RejectSlabSummary, RejectSlabMerkle | Logic | Policy violation or slab validation failure |
| E005 | — | Malformed | Bad request JSON |

---

## HTTP Status Mapping

| Scenario | HTTP Status | Body Has Error? |
|----------|-----------|--------------|
| ACCEPT | 200 | No |
| REJECT (verifier) | 200 | Yes |
| MALFORMED | 400 | Yes |
| ERROR | 500 | Yes |

---

## UnifiedResponse Schema

```typescript
interface UnifiedResponse<T> {
  request_id: string;      // UUID for tracing
  coh_version: string;    // "0.1.0"
  status: Decision;      // "Accept" | "Reject"
  data: T | null;     // Response payload
  error: ApiError | null;
}

interface ApiError {
  code: string;       // E001-E005
  message: string;    // Human-readable
  request_id: string;
}
```

---

## Client Example (Python)

```python
import requests

BASE_URL = "http://localhost:3030"

def verify_chain(receipts):
    resp = requests.post(f"{BASE_URL}/v1/verify-chain", json={
        "receipts": receipts
    })
    body = resp.json()
    
    if body.get("error"):
        print(f"Rejected: {body['error']['code']}")
        print(f"Message: {body['error']['message']}")
        return False
    
    print("Accepted!")
    return True

# Usage
receipts = [...]  # Load from JSONL
verify_chain(receipts)
```

Reference: `plans/INTERFACE_BEHAVIOR_MATRIX.md`, `plans/ERROR_REJECT_CONTRACT.md`