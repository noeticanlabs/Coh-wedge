# Receipt Schema Specification

> Formal JSON schema definitions for Coh receipts

## Overview

Coh uses two primary receipt types:
- **MicroReceipt**: Single-step verification
- **SlabReceipt**: Aggregated multi-step summary

---

## MicroReceipt

Single AI agent step verification.

### JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": [
    "schema_id",
    "version",
    "object_id",
    "canon_profile_hash",
    "policy_hash",
    "step_index",
    "state_hash_prev",
    "state_hash_next",
    "chain_digest_prev",
    "chain_digest_next",
    "metrics"
  ],
  "properties": {
    "schema_id": {
      "type": "string",
      "const": "coh.receipt.micro.v1"
    },
    "version": {
      "type": "string",
      "const": "1.0.0"
    },
    "object_id": {
      "type": "string",
      "description": "Agent/workflow identifier"
    },
    "canon_profile_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 hash of canonicalization profile"
    },
    "policy_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 hash of accounting policy"
    },
    "step_index": {
      "type": "integer",
      "minimum": 0,
      "description": "Monotonic step number in chain"
    },
    "state_hash_prev": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 of prior state"
    },
    "state_hash_next": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 of resultant state"
    },
    "chain_digest_prev": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 of prior chain tip"
    },
    "chain_digest_next": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "SHA-256 of current chain tip"
    },
    "metrics": {
      "$ref": "#/$defs/Metrics"
    }
  }
}
```

### Metrics Sub-schema

```json
{
  "$defs": {
    "Metrics": {
      "type": "object",
      "required": ["v_pre", "v_post", "spend", "defect"],
      "properties": {
        "v_pre": {
          "type": "string",
          "pattern": "^[0-9]+$",
          "description": "Value/risk before action"
        },
        "v_post": {
          "type": "string", 
          "pattern": "^[0-9]+$",
          "description": "Value/risk after action"
        },
        "spend": {
          "type": "string",
          "pattern": "^[0-9]+$",
          "description": "Operational cost consumed"
        },
        "defect": {
          "type": "string",
          "pattern": "^[0-9]+$",
          "description": "Uncertainty/variance allowed"
        }
      }
    }
  }
}
```

### Example

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

---

## SlabReceipt

Aggregated multi-step summary with Merkle root.

### JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": [
    "schema_id",
    "version",
    "object_id",
    "canon_profile_hash",
    "policy_hash",
    "range_start",
    "range_end",
    "micro_count",
    "chain_digest_prev",
    "chain_digest_next",
    "state_hash_first",
    "state_hash_last",
    "merkle_root",
    "summary"
  ],
  "properties": {
    "schema_id": {
      "type": "string",
      "const": "coh.receipt.slab.v1"
    },
    "version": {
      "type": "string",
      "const": "1.0.0"
    },
    "object_id": {
      "type": "string"
    },
    "canon_profile_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "policy_hash": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "range_start": {
      "type": "integer",
      "minimum": 0
    },
    "range_end": {
      "type": "integer",
      "minimum": 0
    },
    "micro_count": {
      "type": "integer", 
      "minimum": 1
    },
    "chain_digest_prev": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "chain_digest_next": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "state_hash_first": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "state_hash_last": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "merkle_root": {
      "type": "string",
      "pattern": "^[0-9a-f]{64}$",
      "description": "Merkle tree root of all receipts"
    },
    "summary": {
      "$ref": "#/$defs/SlabSummary"
    }
  }
}
```

### Summary Sub-schema

```json
{
  "$defs": {
    "SlabSummary": {
      "type": "object",
      "required": ["total_spend", "total_defect", "v_pre_first", "v_post_last"],
      "properties": {
        "total_spend": {
          "type": "string",
          "pattern": "^[0-9]+$"
        },
        "total_defect": {
          "type": "string",
          "pattern": "^[0-9]+$"
        },
        "v_pre_first": {
          "type": "string",
          "pattern": "^[0-9]+$"
        },
        "v_post_last": {
          "type": "string",
          "pattern": "^[0-9]+$"
        }
      }
    }
  }
}
```

### Example

```json
{
  "schema_id": "coh.receipt.slab.v1",
  "version": "1.0.0",
  "object_id": "agent.workflow.demo",
  "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
  "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "range_start": 0,
  "range_end": 2,
  "micro_count": 3,
  "chain_digest_prev": "0000000000000000000000000000000000000000000000000000000000000000",
  "chain_digest_next": "d83cdb09d502855e4f4181e1ee9c1b1d11d2951d2857d335fa1ed01d877350c9",
  "state_hash_first": "1111111111111111111111111111111111111111111111111111111111111111",
  "state_hash_last": "3333333333333333333333333333333333333333333333333333333333333333",
  "merkle_root": "3b0e13328732f6f2c3b7680b15345a4399dfd151234ee4ce8d9a2bca587b6aa8",
  "summary": {
    "total_spend": "42",
    "total_defect": "1",
    "v_pre_first": "100",
    "v_post_last": "55"
  }
}
```

---

## Field Meanings

| Field | Domain Concept | Validation |
|-------|--------------|------------|
| `v_pre` | Unresolved Risk | Value before action |
| `v_post` | Remaining Risk | Value after action |
| `spend` | Operational Cost | Capital/work consumed |
| `defect` | Uncertainty Slack | Allowed variance |

**Policy Inequality:**
```
v_post + spend ≤ v_pre + defect
```

For valid receipts: v_post must not exceed remaining risk pool (v_pre - defect) plus what was spent.

---

## Versioning

- **schema_id**: `"coh.receipt.micro.v1"` / `"coh.receipt.slab.v1"`
- **version**: `"1.0.0"`

Both must match exactly or verification rejects with `RejectSchema`.

---

## Hex Format

All hash fields are **64-character lowercase hex** (SHA-256 output):
- `canon_profile_hash`
- `policy_hash`
- `state_hash_prev`, `state_hash_next`
- `chain_digest_prev`, `chain_digest_next`
- `merkle_root`

Invalid hex → `RejectNumericParse`

Reference: `plans/ERROR_REJECT_CONTRACT.md` for reject codes.