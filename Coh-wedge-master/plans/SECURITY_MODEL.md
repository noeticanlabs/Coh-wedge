# Coh Security Model

> Complete threat model and security architecture documentation for the Coh verification system

## 1. Executive Summary

The Coh verification system provides mathematically proven protection against invalid state transitions in autonomous agent workflows. The core security property is straightforward: **any proposal that fails verification cannot change system state**. This document details the trust model, security properties, threat enumeration, and mitigations that make this guarantee hold in practice.

The security model assumes fully adversarial inputs—the APE (Adversarial Proposal Engine) can generate arbitrary proposals, and external inputs should be treated as untrusted until verified. The verification kernel itself is a deterministic, pure function with no external dependencies, randomness, or hidden state.

## 2. Trust Model

### 2.1 Trusted Components

The following components are trusted to behave correctly and are protected by formal verification:

#### 2.1.1 Coh Verification Kernel

The core verification engine (`coh-core`) consists of deterministic pure functions that perform:

- Schema validation
- Canonical encoding verification
- Chain digest continuity validation
- State hash link validation
- Accounting law enforcement
- Policy check enforcement

**Trust basis**: Formal proofs in Lean (`coh-t-stack`) provide mathematical certainty of correctness. The Rust implementation is traced to these proofs via the traceability matrix.

#### 2.1.2 Canonical Serialization (JCS)

JSON Canonicalization (JCS) provides deterministic encoding with these properties:

- No floating-point values
- No locale-dependent ordering
- Stable key ordering (alphabetical)
- No undefined or implementation-dependent behavior

**Trust basis**: JCS is an established standard (RFC 8785). Verification uses canonical bytes, so any serializer mismatch is detected at the boundary.

#### 2.1.3 Reject Code Taxonomy

The complete family of rejection codes is closed and documented in [`ERROR_REJECT_CONTRACT.md`](ERROR_REJECT_CONTRACT.md). Every possible rejection path is known:

| Reject Code | Category |
|------------|----------|
| `RejectSchema` | Input validation |
| `RejectCanonProfile` | Input validation |
| `RejectChainDigest` | Linkage verification |
| `RejectStateHashLink` | Linkage verification |
| `RejectNumericParse` | Input validation |
| `RejectOverflow` | Mathematical validation |
| `RejectPolicyViolation` | Policy enforcement |
| `RejectSlabSummary` | Slab verification |
| `RejectSlabMerkle` | Slab verification |
| `RejectIntervalInvalid` | Linkage verification |

**Trust basis**: All reject paths are enumerated in the code; there are no hidden rejection modes.

#### 2.1.4 Hash Functions

Domain-separated SHA-256 hashing provides cryptographic integrity:

```rust
chainUpdate(cp, prev, bytes) = cp.hashBytes(cp.domainTag ++ prev ++ bytes)
```

**Trust basis**: SHA-256 is collision-resistant. Domain tags prevent cross-profile attacks.

### 2.2 Untrusted Components

These components are assumed to be controlled by an adversary:

#### 2.2.1 APE (Adversarial Proposal Engine)

The APE generates proposals and can create:

- Valid proposals (correct behavior)
- Invalid schema IDs
- Modified chain digests
- Oversized spend amounts
- Malformed numerical values
- Any arbitrary input

**Trust basis**: None. All APE output must be verified before use.

#### 2.2.2 External Receipts

Receipts from external sources (agents, other systems) must be treated as untrusted:

- May have modified schema_id
- May have modified canonical encoding
- May have broken chain links
- May violate accounting laws
- May exceed policy limits

**Trust basis**: None. All external receipts must pass verification.

#### 2.2.3 State Store

The persistence layer is assumed untrusted:

- May return incorrect state
- May have tampered records
- May have missing records

**Trust basis**: State integrity is verified via chain digest verification. Tampering is detected when verifying the chain.

### 2.3 Trust Boundary

```
+=====================================================================+
|                         TRUST BOUNDARY                               |
|                                                                      |
|   +-------------------------------------------------------------+   |
|   |                  COH VERIFICATION KERNEL                    |   |
|   |                                                              |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | Schema Check    | -> | RejectSchema                |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | Canon Check     | -> | RejectCanonProfile        |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | Chain Continuity| -> | RejectChainDigest          |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | State Hash Link | -> | RejectStateHashLink       |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | Accounting Law | -> | RejectPolicyViolation     |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |   | Policy Check   | -> | RejectPolicyViolation   |    |   |
|   |   +------------------+    +-----------------------------+    |   |
|   |         |                                                     |   |
|   |         v                                                     |   |
|   |     ACCEPT                                                  |   |
|   |         |                                                     |   |
|   |         v                                                     |   |
|   |   +------------------+                                        |   |
|   |   | Execution Layer| <- Only runs after ACCEPT              |    |   |
|   |   +------------------+                                        |   |
|   |                                                              |   |
|   +-------------------------------------------------------------+   |
|                         ^                                          |
|                         |                                          |
|   Receipts              |              Decision                      |
|   (untrusted input)     |              (ACCEPT or REJECT)          |
|                                                                      |
+=====================================================================+
```

**Critical invariant**: The execution layer only runs after verification returns ACCEPT. There is no code path that executes a proposal without verification.

## 3. Security Properties

### 3.1 Deterministic Verification

**Property**: For any fixed receipt, policy, and configuration, the verification decision is always the same.

```
verify(receipt, policy, config) = deterministic(result)
```

**Formal statement**: The Lean formalization `Coh.Contract.Verified` proves that given the same inputs, verification always returns the same decision.

**Implications**:

- No timing attacks (verification time is independent of input values)
- No retry attacks (cannot attempt multiple times to find a valid path)
- No dependency on external services
- No randomness in the decision path

**Implementation requirements**:

- No `rand()` or similar in verification path
- No wall-clock time dependencies
- No network calls during verification
- No file I/O during verification (configuration loaded once)

### 3.2 State Safety

**Property**: A REJECT decision never changes system state.

```
State(t) --[REJECT]--> State(t)  // Unchanged
```

**Formal statement**: The execution layer only applies state changes after verification returns ACCEPT. If verification returns REJECT, the state transition function is never called.

**Implications**:

- Invalid proposals cannot affect the system
- Overspend attempts are blocked
- State rollback is impossible via invalid proposals
- Chain hijacking is detected

**Proof**: The `Coh.Kernel.Verifier` module in the Lean formalization explicitly proves that state application only occurs after ACCEPT.

### 3.3 Complete Mediation

**Property**: Every state change must pass through the Coh verification kernel.

```
For all State changes:
   exists receipt, policy, config:
      verify(receipt, policy, config) = ACCEPT
```

**Implications**:

- No bypass code paths
- Noemergency overrides
- No admin shortcuts that skip verification
- No "force" flags

**Proof**: The `Coh.Category.GovCat` module formalizes that all morphisms go through verification.

### 3.4 Non-Repudiation

**Property**: Each valid receipt contains cryptographic evidence of the state transition it represents.

**Evidence types**:

| Evidence | Purpose |
|----------|---------|
| `chain_digest_prev` | Previous digest in chain |
| `chain_digest_next` | Current digest |
| `state_hash_prev` | Previous state hash |
| `state_hash_next` | Current state hash |
| `canon_profile_hash` | Canon profile used |
| Domain tag | Profile identifier |

**Implications**:

- Any accepted transition can be audited
- Rejected transitions can be traced
- Forgery is detectable
- State history is verifiable

### 3.5 Fail-Safe Defaults

**Property**: Any unrecognized input or unexpected condition results in REJECT.

**Implementation**:

- Schema validation fails → REJECT
- Unknown reject code → REJECT
- Parsing error → MALFORMED (not ACCEPT)
- Missing required fields → REJECT

**Implications**:

- New attack vectors default to rejection
- Missing features cannot be exploited
- Configuration errors favor safety

## 4. Threat Model

### 4.1 Attack Surface Overview

The attack surface includes all inputs to the verification system:

```
[APE Generated Proposals] --> [Receipt Builder] --> [Coh Verifier]
                                        ^
[External Receipts] ----------------------|
```

### 4.2 Enumerated Attack Vectors

Each attack vector maps directly to a reject code:

| Attack Vector | Reject Code | Description |
|--------------|-------------|-------------|
| Schema manipulation | `RejectSchema` | Modified schema_id or version |
| Canon profile attack | `RejectCanonProfile` | Mismatched canonical profile |
| Chain digest injection | `RejectChainDigest` | Broken chain continuity |
| State hash injection | `RejectStateHashLink` | State discontinuity |
| Numeric overflow | `RejectOverflow` | Arithmetic overflow |
| Accounting violation | `RejectPolicyViolation` | Overspending |
| Slab tampering | `RejectSlabSummary`, `RejectSlabMerkle` | Macro receipt manipulation |
| Temporal manipulation | (handled by chain) | Time-based attacks |
| Policy bypass | `RejectPolicyViolation` | Policy rule violations |

### 4.3 Adversary Capabilities

**Assumed capabilities** (adversary can do these):

- Generate any proposal via APE
- Modify receipts in transit (if no transport security)
- Send many proposals rapidly
- Craft malicious numerical values
- Attempt to exploit edge cases

**NOT assumed capabilities** (adversary cannot do these):

- Break SHA-256 collision resistance
- Break JCS canonicalization
- Modify verified state hashes
- Bypass the verification kernel
- Predict random values (none used)

### 4.4 Attack Scenarios

#### 4.4.1 Double-Spend Attack

**Scenario**: An agent attempts to spend the same authority twice.

**Attack path**:

1. Create receipt A: spend 50 from authority 100
2. Create receipt B: spend 50 from authority 100 (same as A)
3. Submit both receipts

**Defense**: Chain digest continuity requires B's `chain_digest_prev` to equal A's `chain_digest_next`. The second receipt fails with `RejectChainDigest`.

#### 4.4.2 State Rollback Attack

**Scenario**: An attacker attempts to revert state to a previous value.

**Attack path**:

1. Modify `state_hash_prev` to an old value
2. Submit receipt with modified state hash

**Defense**: `state_hash_prev` must match the stored `state_hash_next` from the previous step. Modifications are detected with `RejectStateHashLink`.

#### 4.4.3 Chain Hijacking Attack

**Scenario**: An attacker attempts to insert a receipt into an existing chain at a different point.

**Attack path**:

1. Modify `chain_digest_prev` to point to a different chain
2. Submit modified receipt

**Defense**: Chain digest continuity is maintained through the entire chain. Invalid links cause `RejectChainDigest`.

#### 4.4.4 Overflow Attack

**Scenario**: An attacker attempts to cause arithmetic overflow.

**Attack path**:

1. Set values close to u128::MAX
2. Add spend amounts to cause overflow

**Defense**: Explicit overflow checks return `RejectOverflow` before any computation would overflow.

#### 4.4.5 Policy Exploitation

**Scenario**: An attacker attempts to find gaps in policy enforcement.

**Attack path**:

1. Submit receipt that barely violates policy
2. Find edge cases in policy logic

**Defense**: All reject paths are enumerated; policy violations are one-to-one with reject codes.

#### 4.4.6 Slab Manipulation

**Scenario**: An attacker attempts to manipulate aggregated receipts.

**Attack path**:

1. Create valid individual receipts
2. Modify summary in macro slab
3. Submit invalid slab

**Defense**: Summary and Merkle root are re-computed and compared. Mismatches cause `RejectSlabSummary` or `RejectSlabMerkle`.

### 4.5 Composition Attacks

Multiple attacks composed together require all individual defenses to pass:

```
[Schema Check] -> [Canon Check] -> [Chain Continuity] -> ...
        |             |                  |
     REJECT         REJECT              REJECT
```

If any single check fails, the entire proposal is rejected.

## 5. Mitigations

### 5.1 Defense Layers

Each verification layer provides independent defense:

1. **Schema validation** - Rejects malformed inputs
2. **Canonical validation** - Ensures consistent encoding
3. **Chain continuity** - Maintains chain integrity
4. **State hash links** - Maintains state continuity
5. **Accounting law** - Enforces conservation
6. **Policy checks** - Enforces business rules

Each layer is independent; failure at any layer causes complete rejection.

### 5.2 Implementation Mitigations

#### 5.2.1 No Partial Acceptance

**Implementation**: Verification either accepts or rejects the entire receipt. There is no partial acceptance.

```rust
fn verify(receipt, policy) -> Decision {
    if !verifySchema(receipt) { return REJECT; }
    if !verifyCanon(receipt) { return REJECT; }
    if !verifyChain(receipt) { return REJECT; }
    if !verifyState(receipt) { return REJECT; }
    if !verifyAccounting(receipt) { return REJECT; }
    if !verifyPolicy(receipt) { return REJECT; }
    ACCEPT
}
```

#### 5.2.2 Fail-Closed Defaults

**Implementation**: Any unrecognized condition returns REJECT.

```rust
fn verifySchema(receipt) -> bool {
    match receipt.schema_id {
        "coh.receipt.micro.v1" => true,
        "coh.receipt.micro.v2" => true,
        _ => false  // Fail-closed: unknown schema is rejected
    }
}
```

#### 5.2.3 Explicit Overflow Checks

**Implementation**: All arithmetic uses checked operations.

```rust
fn applyAction(state, action) -> Result<State, Error> {
    // Explicit overflow check
    let new_value = state.value.checked_add(action.spend)?;
    Ok(State { value: new_value, ..state })
}
```

### 5.3 Formal Verification

The Lean formalization (`coh-t-stack`) provides mathematical proofs of:

- Verification correctness
- State safety
- Chain integrity
- Accounting law enforcement

The traceability matrix documents proof-to-implementation mapping.

## 6. Security Considerations for Deployment

### 6.1 Transport Security

**Recommendation**: Sign receipts when transmitting over networks.

**Options**:

- TLS for transport
- Ed25519 signature on receipts
- JWT bearer tokens

**Rationale**: While the verifier checks internal integrity, transport security prevents man-in-the-middle modifications.

### 6.2 State Persistence

**Recommendation**: Use authenticated state stores.

**Options**:

- Database with access controls
- Append-only logs
- Hardware security modules

**Rationale**: State integrity is verified via chain, but preventing tampering reduces attack surface.

### 6.3 Logging and Auditing

**Recommendation**: Log all verification decisions with full context.

**Logged fields**:

```json
{
  "decision": "REJECT",
  "code": "RejectPolicyViolation",
  "message": "...",
  "step_index": 2,
  "object_id": "agent.workflow.demo",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Rationale**: Audit trails enable post-incident analysis and compliance.

### 6.4 Key Management

**Recommendation**: Protect canon profiles and policy configurations.

**Requirements**:

- Secure storage for policy keys
- Access controls on configuration
- Rotation schedules

**Rationale**: Policy configuration controls verification behavior.

## 7. Limitations and Assumptions

### 7.1 Cryptographic Assumptions

- SHA-256 collision resistance holds
- JCS canonicalization is stable across implementations
- No cryptographic breaks in the hash functions

### 7.2 Implementation Assumptions

- No floating-point arithmetic in verification path
- No external dependencies during verification
- No wall-clock time in decision path
- No randomness in decision path
- Configuration is loaded once at startup

### 7.3 Known Limitations

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| State size bounded by u128 | Cannot handle values > u128::MAX | Use smaller values or explicit overflow checks |
| Merkle tree depth limited | Cannot handle arbitrarily long chains | Configure maximum depth |
| Policy scope limited | Only current policies supported | Update policy configuration |
| No time-based validity | Receipts don't expire | Implement external expiration |
| Single-chain continuity | Cannot fork/merge chains | Use longer chains externally |

### 7.4 Out of Scope

| Area | Reason |
|------|--------|
| Client-side security | Verifier is server-side |
| Transport encryption | Use TLS + signatures |
| Key management | Use HSM/vault |
| User authentication | External to verifier |
| Rate limiting | External to verifier |

## 8. Compliance Mapping

### 8.1 Security Property Mapping

| Security Property |对应 Standard |
|-----------------|-------------|
| Deterministic verification | Reproducibility |
| State safety | Fail-safe |
| Complete mediation | MAC / verified paths |
| Non-repudiation | Accountability |
| Formal verification | High assurance |

### 8.2 Audit Trail Requirements

| Requirement | Implementation |
|------------|--------------|
| Decision logging | JSON structured logs |
| Reject code tracking | Per-code counters |
| Step index tracking | Included in reject output |
| Object ID tracking | Included in output |

---

## Appendix: Related Documents

- [`ERROR_REJECT_CONTRACT.md`](ERROR_REJECT_CONTRACT.md) - Complete reject code taxonomy
- [`SYSTEM_ARCHITECTURE.md`](coh-node/SYSTEM_ARCHITECTURE.md) - System flow and component mapping
- [`LEAN_RUST_TRACEABILITY_MATRIX.md`](LEAN_RUST_TRACEABILITY_MATRIX.md) - Formal proof mapping
- [`CONTRACT_LAYER_FREEZE_PLAN.md`](CONTRACT_LAYER_FREEZE_PLAN.md) - Contract boundary specification
- [`COMPREHENSIVE_IMPROVEMENT_PLAN.md`](COMPREHENSIVE_IMPROVEMENT_PLAN.md) - Threat model roadmap

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-01-15 | Initial document |