# Case Study: AI Financial Reconciliation Failure

## The $400,000 Hallucination

> *A real-world scenario modeled after common AI agent deployment failures in financial services.*

---

## Background

A mid-size lending firm deployed an AI agent to perform nightly financial reconciliation across 12 regional ledgers. The agent used a large language model to process transaction logs, identify discrepancies, and generate reconciliation reports.

The system ran without incident for 47 days.

On day 48, the agent hallucinated.

---

## The Failure

### What Happened (Without Coh)

**Step 24 — Normal operation**
The agent processed the day's final batch:
- Opening balance (v_pre): $1,200,000
- Transactions processed (spend): $12,000
- Closing balance (v_post): $1,188,000

The accounting law holds: `v_post + spend = v_pre` ✓

**Step 25 — The hallucination**

The LLM, confused by a malformed input from a regional ledger, produced a completion that reported:
- Opening balance (v_pre): $1,188,000
- Transactions processed (spend): $1 (claimed trivially small)
- Closing balance (v_post): $1,588,000 ← **phantom $400,000**

The accounting law is violated: `v_post + spend > v_pre + defect`  
`1,588,000 + 1 = 1,588,001 > 1,188,000 + 0`

**But there was no validator to catch it.**

The agent committed this state to the database and continued.

**Steps 26–10,000 — Cascading corruption**

Every subsequent calculation used the corrupted $1,588,000 balance as its starting point. Over the next 9,975 steps, the phantom value propagated across 12 regional ledgers, compounding interest calculations, altering risk assessments, and generating regulatory reports based on false data.

---

## The Timeline of Damage

| Time | Event |
|---|---|
| Day 48, 02:31 AM | Hallucination occurs at step 25 |
| Day 48, 02:31 AM – Day 51, 09:00 AM | Corrupted state silently propagates |
| Day 51, 09:15 AM | Junior analyst notices balance discrepancy during manual spot-check |
| Day 51 – Day 53 | 3 days of manual forensic accounting to isolate root cause |
| Day 53 | Regulatory body notified of material misstatement |
| Week 8 | $2.1M fine + $400K corrective adjustment + 6 months of remediation |

**Total cost: $2.5M + reputational damage + regulatory scrutiny**

---

## The Same Scenario — With Coh Validator

**Step 24 — Normal operation (identical)**  
Coh verifies the receipt. Policy holds. Step committed. Receipt cryptographically chained.

```
Step 24: ACCEPT  v_pre=1200000  spend=12000  v_post=1188000  [13μs]
```

**Step 25 — Hallucination attempt**

The agent produces the same malformed output. The adapter layer generates a `MicroReceiptWire` encoding the claimed values. Coh evaluates:

```
CHECKING:  v_post + spend ≤ v_pre + defect
COMPUTED:  1,588,000 + 1 = 1,588,001
BOUND:     1,188,000 + 0 = 1,188,000
RESULT:    1,588,001 > 1,188,000  ← VIOLATION
```

```
╔══════════════════════════════════════════════════════════════╗
║              ⚡  CIRCUIT BREAKER TRIGGERED  ⚡               ║
╚══════════════════════════════════════════════════════════════╝

  BREACH AT STEP:   25
  VIOLATION:        v_post + spend  >  v_pre + defect
  ARITHMETIC:       1588000 + 1 = 1588001  >  1188000 + 0 = 1188000
  REJECT CODE:      RejectPolicyViolation
  DETECTION TIME:   13ms
  DAMAGE:           $0 — state never committed
```

The agent is halted. The corrupted value is never written to the database. Steps 26–10,000 never execute.

**An alert is sent to the on-call engineer.**

---

## Side-by-Side Outcome

| Metric | Without Coh | With Coh |
|---|---|---|
| Detection latency | 3 days | **13ms** |
| Corrupted state written | Yes (9,975 steps) | **None** |
| Financial damage | $400K phantom + $2.1M fine | **$0** |
| Steps contaminated | 9,975 | **0** |
| Audit trail | None | **Cryptographic chain** |
| Recovery time | 6 months | **Minutes** |
| False positives | N/A | **Zero (deterministic)** |

---

## How to Replicate This Scenario

Run the included showcase demo to see the circuit breaker in action:

```bash
cargo run --example showcase -p coh-core --release
```

Or integrate Coh into your own agent loop using the template in  
[`examples/integrations/generic_agent_loop.rs`](examples/integrations/generic_agent_loop.rs).

---

## Technical Summary

The Coh Validator enforces the **accounting law of transitions**:

```
v_post + spend ≤ v_pre + defect
```

This single inequality is formally proven in Lean 4 (see [coh-lean](https://github.com/noeticanlabs/coh-lean)) and mechanically enforced at runtime using:

- **u128 exact-integer arithmetic** — no floating-point drift
- **SHA-256 chain digest** — cryptographic tamper detection
- **Deterministic canonicalization** — reproducible across environments
- **Sub-millisecond latency** — ~13μs per step in release mode

The system is provably correct by construction: if the accounting law holds for every step, the aggregate slab also holds. This is the `lawful_composition` theorem in the formal specification.

---

*Built with rigor by the Antigravity Team — NoeticanLabs*  
*Formal verification: [github.com/noeticanlabs/coh-lean](https://github.com/noeticanlabs/coh-lean)*
