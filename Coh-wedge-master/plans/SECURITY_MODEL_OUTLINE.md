# SECURITY_MODEL.md - Content Outline

## 1. Executive Summary

One-paragraph overview of the security model, emphasizing: deterministic verification, state safety guarantees, and the core security property: "REJECT means state remains unchanged."

## 2. Trust Model

### 2.1 Trusted Components

- **Coh Verification Kernel**: Deterministic pure functions; mathematically proven correctness (Lean formalization)
- **Canonical Serialization (JCS)**: Stable, deterministic encoding; no floats, locale dependence, or hidden state
- **Reject Code Taxonomy**: Closed family; all rejection paths are known and enumerated

### 2.2 Untrusted Components

- **APE (Adversarial Proposal Engine)**: Generates arbitrary proposals; assumed adversarial
- **External Inputs**: Receipts from external sources; must be verified
- **State Store**: Persistence layer; integrity verified via hash chains

### 2.3 Trust Boundary

```
┌─────────────────────────────────────────────────────────────┐
│                     Trust Boundary                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              COH VERIFICATION KERNEL                │   │
│  │  - Schema validation                                 │   │
│  │  - Canonical verification                             │   │
│  │  - Chain continuity (state/chain digest)             │   │
│  │  - Accounting law (inequality check)                │   │
│  │  - Policy enforcement                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                         ↑                                 │
│   Receipts (untrusted input) → Decision (ACCEPT/REJECT)  │
└─────────────────────────────────────────────────────────────┘
```

## 3. Security Properties

### 3.1 Deterministic Verification

- **Property**: Same receipt + same policy → always same decision
- **Implication**: No randomness, no timing attacks, no dependency on external services
- **Implementation**: Pure functions in Rust; formal proof in Lean

### 3.2 State Safety

- **Property**: REJECT never changes system state
- **Formal**: `State(t) → [REJECT] → State(t)` (unchanged)
- **Implication**: Invalid proposals cannot affect the state

### 3.3 Complete Mediation

- **Property**: Every state change must pass through Coh verification
- **Implication**: No bypass possible; execution layer only runs post-ACCEPT

### 3.4 Non-Repudiation (per Receipt)

- **Property**: Each valid receipt contains cryptographic evidence of the transition
- **Implementation**: Chain digest, state hash links, domain-separated hashing

## 4. Threat Model

### 4.1 Enumerated Attack Vectors

Each reject code maps to a specific attack type:

| Attack Vector | Reject Code | Description |
|--------------|-------------|-------------|
| Schema tampering | `RejectSchema` | Modified schema_id or version |
| Canon profile attack | `RejectCanonProfile` | Modified canonical profile parameters |
| Chain digest injection | `RejectChainDigest` | Broken chain continuity |
| State hash injection | `RejectStateHashLink` | Modified prior state |
| Numeric overflow | `RejectOverflow` | Arithmetic overflow attempt |
| Accounting violation | `RejectPolicyViolation` | Spending beyond authority |
| Slab tampering | `RejectSlabSummary`, `RejectSlabMerkle` | Macro receipt manipulation |

### 4.2 Adversary Capabilities

- **Assumed**: Full control of APE, can generate any proposal
- **Assumed**: Can modify receipts in transit (if no transport security)
- **NOT Assumed**: Cannot break cryptographic primitives (SHA-256, JCS)
- **NOT Assumed**: Cannot modify verified state or chain digests

### 4.3 Attack Scenarios

1. **Double-spend attack**: Attempt to spend same authority twice → `RejectPolicyViolation`
2. **State rollback**: Modify state_hash_prev → `RejectStateHashLink`
3. **Chain hijack**: Modify chain_digest_prev → `RejectChainDigest`
4. **Overflow attack**: Craft numbers to exceed u128::MAX → `RejectOverflow`
5. **Schema downgrade**: Use old/vulnerable schema → `RejectSchema`

## 5. Mitigations

### 5.1 Input Validation Layers

1. Schema validation (first gate)
2. Canonical encoding validation
3. Hash link verification
4. Accounting law validation
5. Policy validation

### 5.2 Defense in Depth

- Each layer is independent; failure at any layer causes REJECT
- No partial acceptance; complete verification required

### 5.3 Formal Verification

- Lean proofs for core security properties
- Traceability matrix documenting proof-to-implementation mapping

## 6. Security Considerations for Deployment

### 6.1 Transport Security

-Receipts should be signed if transmitted over network
- Verify signature before processing

### 6.2 State Persistence

- State store integrity protected by chain digest
- Tampering detected via hash chain validation

### 6.3 Logging & Auditing

- All decisions logged with full trace
- REJECT reasons provide audit trail

## 7. Limitations & Assumptions

### 7.1 Cryptographic Assumptions

- SHA-256 collision resistance
- JCS deterministic encoding stability

### 7.2 Implementation Assumptions

- No floating-point in verification path
- No external system dependencies in verifier
- No wall-clock time or RNG in decision path

### 7.3 Known Limitations

- State size bounded by u128
- Merkle tree depth limited (configuration)
- Policy scope limited to current policy set

## 8. Compliance Mapping

| Security Property | Maps to |
|-----------------|---------|
| Deterministic verification | Reproducibility requirement |
| State safety | Fail-safe behavior |
| Complete mediation | Mandatory access control |
| Non-repudiation | Accountability |
| Formal verification | High assurance certification |

---

## Appendix: Related Documents

- [ERROR_REJECT_CONTRACT.md](./ERROR_REJECT_CONTRACT.md) - Reject code taxonomy
- [SYSTEM_ARCHITECTURE.md](./coh-node/SYSTEM_ARCHITECTURE.md) - System flow
- [LEAN_RUST_TRACEABILITY_MATRIX.md](./LEAN_RUST_TRACEABILITY_MATRIX.md) - Formal proof mapping