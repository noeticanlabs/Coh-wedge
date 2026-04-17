# Execution Layer - Proof Format Specification

> Detailed specification for the execution proof that proves actions happened correctly

---

## Proof Schema

The execution proof follows the `coh.receipt.execution.v1` schema:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "title": "Coh Execution Receipt",
  "description": "Proof that an action was executed after verification",
  "required": [
    "schema_id",
    "version",
    "parent_receipt_hash",
    "action_result",
    "execution_timestamp",
    "state_hash_prev",
    "state_hash_next"
  ],
  "properties": {
    "schema_id": {
      "type": "string",
      "const": "coh.receipt.execution.v1"
    },
    "version": {
      "type": "string",
      "const": "1.0.0"
    },
    "parent_receipt_hash": {
      "type": "string",
      "description": "Hash of the verification receipt that approved this execution",
      "pattern": "^[0-9a-f]{64}$"
    },
    "action_result": {
      "type": "object",
      "required": ["status", "state_prev", "state_next"],
      "properties": {
        "status": {
          "type": "string",
          "enum": ["success", "failed", "blocked"]
        },
        "state_prev": {
          "type": "string",
          "description": "State hash before action execution"
        },
        "state_next": {
          "type": "string",
          "description": "State hash after action execution"
        }
      }
    },
    "execution_timestamp": {
      "type": "integer",
      "description": "Unix timestamp of execution"
    },
    "state_hash_prev": {
      "type": "string",
      "description": "Canonical state hash before execution"
    },
    "state_hash_next": {
      "type": "string",
      "description": "Canonical state hash after execution"
    }
  }
}
```

---

## Proof Example

### Valid Execution Proof

```json
{
  "schema_id": "coh.receipt.execution.v1",
  "version": "1.0.0",
  "parent_receipt_hash": "03e3fb655ac06d124267f0beb32ee7edc6c770571cf3fb48be83f4d704a50127",
  "action_result": {
    "status": "success",
    "state_prev": "0000000000000000000000000000000000000000000000000000000000000000",
    "state_next": "a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd"
  },
  "execution_timestamp": 1700000123,
  "state_hash_prev": "0000000000000000000000000000000000000000000000000000000000000000",
  "state_hash_next": "a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd"
}
```

### Rejected Execution (No Proof)

When execution is blocked, no proof is generated:

```json
{
  "decision": "Reject",
  "error": "RejectPolicyViolation: accounting law violated",
  "execution_proof": null
}
```

---

## State Transition Chain

The proof creates a verifiable chain:

```
State S0
    │
    ├─[Receipt R1]─[Verify]─[Accept]─[Execute]─> State S1
    │                                              │
    │                                              Proof P1:
    │                                              { parent: R1, S0→S1 }
    │                                              │
    └─[Receipt R2]─[Verify]─[Reject]─X             │
        (blocked, no state change)                 │
                                                  ▼
                                                State S1
                                                    │
                                                    ├─[Receipt R3]─[Accept]─> State S2
                                                    │                         │
                                                    │                         Proof P2:
                                                    │                         { parent: R3, S1→S2 }
                                                    │                         │
                                                    └─[Receipt R4]─[Accept]─> State S3
```

---

## Verification of Proof

Third parties can verify the proof:

1. **Parent Receipt Verification**
   - Hash the parent receipt
   - Confirm it matches `parent_receipt_hash`

2. **State Transition Verification**
   - Confirm `state_hash_prev` matches prior state
   - Confirm `state_hash_next` computed correctly from action

3. **Timestamp Verification**
   - Confirm `execution_timestamp` is after `parent_receipt` timestamp
   - Confirm timestamp is within acceptable window

---

## Use Cases

### Audit Trail
```sql
SELECT * FROM execution_proofs 
WHERE object_id = 'work_order_123' 
ORDER BY execution_timestamp;
```

### Compliance Reporting
```json
{
  "report": "monthly_execution_summary",
  "total_executed": 150,
  "total_rejected": 23,
  "proofs": [ ... ]
}
```

### Third-Party Verification
```rust
fn verify_proof(proof: &ExecutionProof) -> bool {
    let parent = fetch_receipt(&proof.parent_receipt_hash);
    let state_valid = verify_state_transition(
        proof.state_hash_prev,
        proof.state_hash_next,
        &proof.action_result
    );
    parent.is_accept() && state_valid
}
```

---

## Storage

### State Store Schema

```rust
struct StateStore {
    // key: object_id
    // value: Vec<StateTransition>
}

struct StateTransition {
    state_prev: String,
    state_next: String,
    proof: ExecutionProof,
    timestamp: u64,
}
```

### Query Examples

```rust
// Get current state
store.get("work_order_123")

// Get state history
store.history("work_order_123")

// Get proof for specific execution
store.get_proof("work_order_123", execution_id)
```

---

## Integration with CLI

```bash
# Execute with proof (dry-run)
coh-validator.exe execute \
  --receipt receipt.json \
  --action action.json \
  --mode dry-run

# Execute with proof (real)
coh-validator.exe execute \
  --receipt receipt.json \
  --action action.json \
  --mode real

# Query state
coh-validator.exe state get work_order_123

# Query proof
coh-validator.exe proof get execution_abc123
```

---

## See Also

- [EXECUTION_LAYER_DESIGN.md](EXECUTION_LAYER_DESIGN.md) - Architecture overview
- [execute.rs](../coh-node/crates/coh-core/src/execute.rs) - Implementation
- [RECEIPT_SCHEMA_SPEC.md](../plans/RECEIPT_SCHEMA_SPEC.md) - Other receipt types