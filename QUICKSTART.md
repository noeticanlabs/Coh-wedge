# End-to-End Formation Walkthrough

This guide walks through the complete **Chaos-Coherence** pipeline from forward generation to backward certification.

---

## 1. Forward Generation: The Law of Chaos

The **Noetic Proposal Engine (NPE)** ensures that every AI-generated candidate satisfies the forward admissibility law:
`M(g') + C(p) <= M(g) + D(p)`

```bash
cargo test -p coh-npe
```

## 2. Backward Certification: The Law of Coherence

The **Verifier Kernel** ensures that every executed step satisfies the backward certification law:
`v_post + spend <= v_pre + defect + authority`

```bash
cargo run --manifest-path coh-node/Cargo.toml -p coh-validator --release -- \
  verify-chain coh-node/vectors/valid/valid_chain_10.jsonl
```

## 3. The Formation Boundary

Formation occurs at the intersection. In the **Coh Dashboard**, you can visualize the "Active Boundary" where these margins are tight.

```bash
cd coh-dashboard
npm run dev
```

---

## Input Formats

### Chaos Candidate (NPE)
```json
{
  "prev_disorder": "100",
  "next_disorder": "95",
  "cost": "10",
  "slack": "5"
}
```

### Micro Receipt (Verifier)
```json
{
  "v_pre": "1000",
  "v_post": "950",
  "spend": "30",
  "defect": "5"
}
```

---

## Reject Codes

| Code | Layer | Meaning |
|------|-------|---------|
| `ChaosViolation` | NPE / Formation | Fails forward generation law |
| `ProjectionMismatch` | Formation | Chaos candidate doesn't match receipt |
| `SemanticEnvelopeViolation` | Formation | Defect fails to dominate δ_hat |
| `RejectPolicyViolation` | Verifier | Fails backward coherence law |
| `RejectChainDigest` | Verifier | Cryptographic linkage broken |

---

## Next Steps

- [ ] Explore [coh-t-stack/](coh-t-stack/) for the **Active Boundary Inclusion Theorem** proof.