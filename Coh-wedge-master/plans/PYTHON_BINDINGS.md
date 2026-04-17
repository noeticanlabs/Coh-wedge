# Coh Python Bindings

> Python API for verification and receipt manipulation

## Installation

```bash
pip install coh-python
# Or from source:
cd coh-node/crates/coh-python
pip install .
```

## Import

```python
import coh
from coh import normalize, verify, verify_chain, build_slab, verify_slab, hash, compare
```

---

## Exceptions

| Exception | Raised When |
|-----------|------------|
| `CohError` | Base exception |
| `CohVerificationError` | Verification rejected |
| `CohMalformedError` | Input parse/format error |

---

## Functions

### normalize(input) -> CohResult

Normalize a receipt to canonical form and compute its digest.

```python
# From JSON string
result = coh.normalize('{"schema_id": "coh.receipt.micro.v1", ...}')

# From dict
result = coh.normalize({
    "schema_id": "coh.receipt.micro.v1",
    "version": "1.0.0",
    "object_id": "agent.workflow.demo",
    "step_index": 0,
    "metrics": {"v_pre": "100", "v_post": "88", "spend": "12", "defect": "0"}
})

print(result.normalized)  # Canonical form
print(result.hash)        # Digest (hex)
```

**Accepts:** JSON string, dict

**Raises:** `CohMalformedError` on parse failure

---

### verify(input)

Verify a single micro-receipt. Raises exception on rejection.

```python
# From JSON string
coh.verify('{"schema_id": "coh.receipt.micro.v1", ...}')

# From dict
coh.verify({
    "schema_id": "coh.receipt.micro.v1",
    "version": "1.0.0",
    "step_index": 0,
    "metrics": {"v_pre": "100", "v_post": "88", "spend": "12", "defect": "0"}
})
# On success: returns None
# On failure: raises CohVerificationError
```

**Accepts:** JSON string, dict

**Raises:** 
- `CohVerificationError` on rejection
- `CohMalformedError` on parse failure

---

### verify_chain(input)

Verify a chain of receipts.

```python
# From JSONL string
result = coh.verify_chain("""{"step_index": 0, ...}
{"step_index": 1, ...}""")

# From list of dicts
result = coh.verify_chain([
    {"step_index": 0, ...},
    {"step_index": 1, ...}
])

print(result.decision)       # "Accept" | "Reject"
print(result.code)         # RejectCode or None
print(result.message)      # Human-readable
print(result.step_index)  # Failing step index (if any)
```

**Accepts:** JSONL string, list

**Returns:** Result object with `.decision`, `.code`, `.message`, `.failing_step_index`

---

### build_slab(input)

Build a slab from a chain.

```python
slab = coh.build_slab([
    {"step_index": 0, ...},
    {"step_index": 1, ...}
])

print(slab.schema_id)     # "coh.receipt.slab.v1"
print(slab.merkle_root)  # Merkle root hash
print(slab.summary)      # {total_spend, total_defect, ...}
```

**Accepts:** JSONL string, list

**Returns:** SlabReceiptWire dict

---

### verify_slab(input)

Verify a standalone slab.

```python
result = coh.verify_slab({
    "schema_id": "coh.receipt.slab.v1",
    "merkle_root": "abc123...",
    "summary": {...}
})

print(result.decision)  # "Accept" | "Reject"
print(result.code)    # RejectCode if rejected
```

**Accepts:** JSON string, dict

---

### hash(input)

Compute SHA-256 digest.

```python
digest = coh.hash({"key": "value"})
# Returns 64-character hex string
```

---

### compare(a, b)

Compare two normalized forms.

```python
equal = coh.compare(form_a, form_b)
# Returns True/False
```

---

### assert_equivalent(a, b)

Assert two normalized forms are equivalent. Raises if not.

```python
coh.assert_equivalent(form_a, form_b)
# On inequality: raises CohVerificationError
```

---

## Usage Examples

### Agent Loop Integration

```python
import coh

def agent_step(state, action, result):
    receipt = {
        "schema_id": "coh.receipt.micro.v1",
        "version": "1.0.0",
        "object_id": "my-agent",
        "step_index": state.step,
        "state_hash_prev": state.hash,
        "state_hash_next": compute_next_hash(state, action),
        "metrics": {
            "v_pre": str(state.value),
            "v_post": str(result.value),
            "spend": str(action.cost),
            "defect": str(result.error)
        }
    }
    
    try:
        coh.verify(receipt)
        # Accept - proceed
    except CohVerificationError as e:
        # Reject - circuit break
        print(f"Verification failed: {e}")
        raise
```

### Batch Verification

```python
# From JSONL file
with open("receipts.jsonl") as f:
    content = f.read()

result = coh.verify_chain(content)
if result.decision == "Accept":
    print("All receipts valid!")
else:
    print(f"Rejected at step {result.failing_step_index}: {result.message}")
```

### Chain + Slab

```python
# Verify chain first
chain_result = coh.verify_chain(chain_receipts)
if chain_result.decision == "Accept":
    # Build slab
    slab = coh.build_slab(chain_receipts)
    
    # Verify slab
    slab_result = coh.verify_slab(slab)
    print(f"Slab valid: {slab_result.decision}")
```

---

## Input Flexibility

All functions accept multiple input forms:

| Function | String (JSON) | String (JSONL) | Dict | List[Dict] |
|----------|--------------|----------------|------|-----------|
| normalize | ✅ | — | ✅ | — |
| verify | ✅ | — | ✅ | — |
| verify_chain | — | ✅ | — | ✅ |
| build_slab | — | ✅ | — | ✅ |
| verify_slab | ✅ | — | ✅ | — |
| hash | ✅ | — | ✅ | — |

---

## Result Objects

### Verification Result

```python
{
    "decision": "Accept" | "Reject",
    "code": RejectCode | None,  # e.g., "RejectPolicyViolation"
    "message": str,
    "step_index": int | None,
    "object_id": str | None
}
```

### CohResult (normalize)

```python
{
    "normalized": dict,   # Canonical form
    "hash": str         # Hex digest
}
```

Reference: `plans/ERROR_REJECT_CONTRACT.md`, `plans/INTERFACE_BEHAVIOR_MATRIX.md`