# Purpose and Scope

## What Is the Coh Validator?

The Coh Validator ("Frozen Wedge") is a deterministic, cryptographically-anchored constraint verifier for AI agent workflows. It enforces the **Accounting Law of Transitions** at every step of an agent execution, detecting hallucinated numeric values, tampered state, and policy violations before they are committed.

It is the reference Rust implementation of the Coh protocol — bridging the formally verified semantics of [coh-lean](https://github.com/noeticanlabs/coh-lean) with real-world deployment.

---

## The Core Invariant

For every micro-receipt, the system enforces:

```
v_post + spend <= v_pre + defect
```

Where:
- `v_pre` — unresolved risk/value before the agent step
- `v_post` — unresolved risk/value after the agent step
- `spend` — operational cost consumed in this step
- `defect` — tolerated slack / allowed variance

Violation results in an immediate `RejectPolicyViolation` with a machine-readable exit code.

---

## Scope of V1 ("Frozen Wedge")

The V1 scope is deliberately narrow and fully locked. It covers exactly:

1. **verify-micro** — single-step receipt verification
2. **verify-chain** — contiguous chain verification with state-linkage checks
3. **build-slab** — chain aggregation into a Merkle-rooted slab receipt
4. **verify-slab** — standalone slab macro-accounting check

V1 does **not** include: network transport, agent SDKs, multi-tenant routing, or streaming ingestion. Those belong to V2.

---

## Design Principles

- **Deterministic by construction**: same input always produces identical output, independent of time or environment.
- **Zero false positives**: the accounting law is mathematical; there are no probabilistic classifiers.
- **Offline-first**: no network calls, no external dependencies at runtime.
- **Formally grounded**: the invariant is proved in Lean 4 (see `07-lean-to-rust-traceability.md`).

