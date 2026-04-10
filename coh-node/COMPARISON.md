# Coh Validator — Comparison Table

> **"The only AI safety tool that enforces a cryptographic accounting law at the micro-step level."**

---

## Feature Matrix

| Capability | Traditional Audit Logs | Checksums | LangChain Guardrails | OpenAI Moderation | **Coh Validator** |
|:---|:---:|:---:|:---:|:---:|:---:|
| Detects value creation from nothing | ❌ | ❌ | ❌ | ❌ | ✅ |
| Cryptographic tamper-proof chain | ❌ | Partial | ❌ | ❌ | ✅ |
| Sub-millisecond latency | ❌ | ✅ | ❌ | ❌ | ✅ (~13μs/step) |
| Formally verified (Lean 4) | ❌ | ❌ | ❌ | ❌ | ✅ |
| Deterministic (same input → same output) | ❌ | ✅ | ❌ | ❌ | ✅ |
| Works offline (no API calls) | ✅ | ✅ | ❌ | ❌ | ✅ |
| Step-by-step verification | ❌ | ❌ | Partial | ❌ | ✅ |
| Detects hallucinated numeric values | ❌ | ❌ | Partial | ❌ | ✅ |
| Merkle-root audit trail | ❌ | ❌ | ❌ | ❌ | ✅ |
| Zero false positives (mathematical) | ❌ | ✅ | ❌ | ❌ | ✅ |
| Machine-readable reject codes | ❌ | ❌ | Partial | ✅ | ✅ |
| Circuit-break on first violation | ❌ | ❌ | Partial | ❌ | ✅ |

---

## Detailed Comparison

### Coh Validator vs. Traditional Audit Logs

**Traditional audit logs** record what happened but cannot prove it is correct.  
A corrupted agent can write plausible-looking logs while violating the underlying accounting invariant.

**Coh Validator** enforces the accounting law _before_ state is committed:

```
v_post + spend ≤ v_pre + defect
```

Every receipt is cryptographically chained, so the log is tamper-evident. If an agent skips a step, rewrites a balance, or hallucinates a value, the chain digest breaks — detected in microseconds, not days.

| | Audit Log | Coh Validator |
|---|---|---|
| Detection latency | 1–3 days (manual review) | ~13ms (automated) |
| Proof method | Human inspection | Mathematical proof |
| Tamper resistance | None | SHA-256 chain |
| Rollback signal | None | `RejectChainDigest` |

---

### Coh Validator vs. Checksums

**Checksums** verify file or message integrity but cannot reason about _semantic correctness_. A checksum confirms a file hasn't changed; it cannot tell you whether the _content_ of that file violates a conservation law.

**Coh Validator** uses a checksum _and_ an accounting law:

- SHA-256 chain digest: ensures no receipt was modified after creation.
- Policy inequality: ensures the numbers inside the receipt are semantically valid.

A checksum would PASS a receipt where `v_post = v_pre + 1,000,000` (value from nothing), because the bytes are internally consistent. **Coh would REJECT it** with `RejectPolicyViolation`.

---

### Coh Validator vs. LangChain Guardrails

**LangChain Guardrails** apply content-level filters (e.g., "don't say harmful things") using heuristics and LLM-based classifiers. They are:

- **Non-deterministic**: The same input can produce different outcomes.
- **Latency-heavy**: Require extra LLM calls (100ms–2s per check).
- **Semantic only**: Cannot detect numeric invariant violations.
- **Not cryptographic**: Produce no tamper-proof receipt.

**Coh Validator** is the complement, not the competitor. Use LangChain Guardrails for content policy; use Coh for _accounting correctness_:

| | LangChain Guardrails | Coh Validator |
|---|---|---|
| Target | Text content safety | Numeric/state integrity |
| Method | LLM classifier | Deterministic arithmetic |
| Latency | 100ms–2s | ~13μs |
| False positives | Yes (probabilistic) | Zero (mathematical) |
| Offline | No | Yes |
| Cryptographic proof | No | Yes |

---

### Coh Validator vs. Other AI Safety Tools

| Tool | Category | Gap vs. Coh |
|---|---|---|
| Constitutional AI (Anthropic) | RLHF alignment | Cannot enforce accounting laws at runtime |
| RLHF reward models | Training-time safety | No runtime verification |
| Automated red-teaming | Security testing | Tests, doesn't prevent |
| Python assert statements | Code-level checks | No cryptographic proof; bypassable |
| Database transactions | Storage integrity | Doesn't verify agent reasoning |

---

## The Unique Value Proposition

Coh Validator occupies a **category of one**: a formally verified, cryptographically anchored, sub-millisecond accounting kernel for AI agent workflows.

```
                    Content Safety
                    (LangChain, OpenAI Moderation)
                           ↑
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
Slow     │    Heuristic    │   Combined      │ Fast
         │    guardrails   │   approach      │
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │
                           ↓
                 Accounting Integrity
                 (Coh Validator)
```

**Coh fills the bottom half**: deterministic, cryptographic, zero-false-positive enforcement of the arithmetic invariants that make AI agent outputs trustworthy.

---

*Formal verification: [github.com/noeticanlabs/coh-lean](https://github.com/noeticanlabs/coh-lean)*
