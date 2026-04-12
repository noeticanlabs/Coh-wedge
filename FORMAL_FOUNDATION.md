# Coh Safety Wedge — Formal Foundation

This document specifies the mathematical and categorical foundations of the Coh system.

---

## The T-Stack Ledger Architecture

Formal verification of the Coh system is structured as a **T-Stack Federated Ledger**. Each "T" module provides a machine-verified proof of a foundational claim, building toward the complete Dirac Inevitability theorem.

### T1: Categorical Foundation
- **Theorem**: `StrictCoh ? Category`
- **Formal Source**: [coh-t-stack/Coh/Ledger/T1_StrictCohCategory.lean](coh-t-stack/Coh/Ledger/T1_StrictCohCategory.lean)
- **Claim**: The admissible fragment of a Strict Coh System (defined by a deterministic verifier and source-target compatibleArrow arrows) satisfies all Small Category axioms.
- **Verification Status**: [PROVED] zero-sorry.

---

## Core Invariant: The Accounting Law of Transitions

The Coh Safety Wedge enforces a discrete trace accounting law at the runtime kernel level.

```
v_post + spend <= v_pre + defect
```

Where:
- `v_pre` = unresolved risk/value before the agent step.
- `v_post` = unresolved risk/value after the agent step.  
- `spend` = operational cost / work consumed.
- `defect` = tolerated uncertainty / slack.

**Rust enforcement**: `crates/coh-core/src/verify_micro.rs`.

---

## Determinism and Integrity

The Coh system guarantees absolute execution determinism:
- **Integer Arithmetic**: `u128` checked math (`safe_add`, `safe_sub`).
- **Canonical Serialization**: JCS-compatible JSON field ordering.
- **Domain Separation**: Contextual tagging for all cryptographic digests.
- **Environment Isolation**: No reliance on RNG, network, or system clock.
