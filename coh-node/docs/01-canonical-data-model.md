# Canonical Data Model

The Coh Validator uses a strict 4-layer data architecture to eliminate floating-point ambiguity, prevent serialization drift, and guarantee that every receipt can be independently re-verified by any conforming implementation.

---

## Layer 1: Wire

All numeric fields are encoded as **decimal strings** in JSON. No floats, no scientific notation.

```json
{
  "schema_id": "coh.receipt.micro.v1",
  "version": "1.0.0",
  "object_id": "agent.task.42",
  "canon_profile_hash": "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09",
  "policy_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "step_index": 1,
  "state_hash_prev": "0000000000000000000000000000000000000000000000000000000000000001",
  "state_hash_next": "0000000000000000000000000000000000000000000000000000000000000002",
  "chain_digest_prev": "d6f3b24b580b5d4b3f3ee683ecf02ef47e42837cc0d7c13daab4e082923a5149",
  "chain_digest_next": "0000000000000000000000000000000000000000000000000000000000000000",
  "metrics": {
    "v_pre": "80",
    "v_post": "60",
    "spend": "20",
    "defect": "0"
  }
}
```

All wire structs use `#[serde(deny_unknown_fields)]` — extra fields are a hard parse error.

---

## Layer 2: Runtime

Wire values are converted to typed Rust structs:

- Numeric strings → `u128` via checked `parse::<u128>()`
- Hex strings → `Hash32([u8; 32])` via `hex::decode` with length validation
- Any conversion failure → `RejectNumericParse` or `RejectChainDigest`

All arithmetic in this layer uses `checked_add`, `checked_sub`, `checked_mul` — overflow is impossible without returning a `RejectOverflow` error.

---

## Layer 3: Prehash

Before hashing, the receipt is projected onto a **prehash view** with fields in **strict alphabetical order** and `chain_digest_next` structurally excluded. This is the canonical form used for digest computation.

```
canon_profile_hash
chain_digest_prev
metrics.defect
metrics.spend
metrics.v_post
metrics.v_pre
object_id
policy_hash
schema_id
state_hash_next
state_hash_prev
step_index
version
```

Alphabetical ordering ensures the canonical JSON bytes are identical across all conforming serializers.

---

## Layer 4: Result

Every operation returns a typed result struct with:

- `decision`: `ACCEPT`, `REJECT`, or `SLAB_BUILT`
- `code`: optional `RejectCode` (machine-readable)
- `message`: human-readable description

Result structs are serializable to JSON via `--format json` for pipeline integration.

---

## Field Reference

| Field | Type (Wire) | Type (Runtime) | Description |
|---|---|---|---|
| `schema_id` | String | String | Fixed: `coh.receipt.micro.v1` |
| `version` | String | String | Fixed: `1.0.0` |
| `object_id` | String | String | Identifies the entity being tracked |
| `canon_profile_hash` | Hex string | Hash32 | Protocol version identifier |
| `policy_hash` | Hex string | Hash32 | Reserved for future policy pinning |
| `step_index` | u64 | u64 | Monotonically increasing step counter |
| `state_hash_prev` | Hex string | Hash32 | Pre-step state commitment |
| `state_hash_next` | Hex string | Hash32 | Post-step state commitment |
| `chain_digest_prev` | Hex string | Hash32 | Previous receipt's chain anchor |
| `chain_digest_next` | Hex string | Hash32 | This receipt's chain anchor (computed) |
| `metrics.v_pre` | Decimal string | u128 | Pre-step potential |
| `metrics.v_post` | Decimal string | u128 | Post-step potential |
| `metrics.spend` | Decimal string | u128 | Work consumed |
| `metrics.defect` | Decimal string | u128 | Allowed slack |
| `metrics.authority` | Decimal string | u128 | Systemic override capacity (default: 0) |

## V3 Schema Additions (Transition Contract)

V3 receipts (`coh.receipt.micro.v3`) extend the base model with additional formation and objective fields:

| Field | Type (Wire) | Type (Runtime) | Description |
|---|---|---|---|
| `objective_result` | Object (Optional) | `ObjectiveResult` | Outcome of the transition objective (`satisfied`, `violated`, `not_applicable`) |
| `sequence_valid` | Boolean | bool | Cryptographic sequence guard |
| `override_applied` | Boolean | bool | Policy override indicator |
| `metrics.m_pre` | Decimal string | u128 | Pre-step mass (Genesis Law) |
| `metrics.m_post` | Decimal string | u128 | Post-step mass (Genesis Law) |
| `metrics.c_cost` | Decimal string | u128 | Cost of proposal (Genesis Law) |
| `metrics.d_slack` | Decimal string | u128 | Slack allowance (Genesis Law) |
| `metrics.projection_hash` | Hex string | Hash32 | Content projection identifier |
| `metrics.pl_tau` | Decimal string | u128 | PhaseLoom temperature/tension |
| `metrics.pl_budget` | Decimal string | u128 | PhaseLoom computation budget |
| `metrics.pl_provenance` | String | String | Strategy provenance identifier |
