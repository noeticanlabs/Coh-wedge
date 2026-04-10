# Roadmap

> **Current release**: V1 "Frozen Wedge" — the locked reference implementation of the Coh protocol.

---

## V1 — Frozen Wedge (Shipped)

The V1 scope is complete and locked. It defines the canonical Coh protocol for single-machine, offline verification of AI agent receipts.

- [x] `verify-micro` — single-step receipt verification (6-step frozen order)
- [x] `verify-chain` — contiguous JSONL chain verification with state-linkage
- [x] `build-slab` — chain aggregation into a Merkle-rooted slab receipt
- [x] `verify-slab` — standalone slab macro-accounting check
- [x] Exit code contract (0–4) for pipeline integration
- [x] `--format json` machine-readable output
- [x] Fixture pack (11 test vectors)
- [x] Digest stability + fixture oracle tests
- [x] Formally verified invariant in Lean 4 (`coh-lean`)
- [x] Integration templates: generic agent loop + OpenAI function calling

---

## V2 — Protocol Hardening

Goal: make Coh production-ready for multi-agent, multi-tenant deployments.

- [ ] **Streamed chain ingestion** — accept receipts incrementally over stdin/socket without buffering the full JSONL file
- [ ] **Receipt signing** — optional Ed25519 signature on each receipt (agent identity anchoring)
- [ ] **Policy pinning** — enforce non-zero `policy_hash` to pin the accounting policy per object
- [ ] **Multi-object chains** — support parallel chains identified by `object_id`, verified independently
- [ ] **Configurable canon profiles** — allow deployers to register custom `canon_profile_hash` values without forking
- [ ] **Structured error codes in JSON output** — machine-readable error metadata beyond current `code` field

---

## V3 — SDK and Ecosystem

Goal: make Coh easy to integrate across languages and platforms.

- [ ] **Rust SDK crate** (`coh-sdk`) — ergonomic builder API for generating receipts from agent callbacks
- [ ] **Python bindings** — PyO3-based wrapper for use with LangChain, LlamaIndex, AutoGen
- [ ] **TypeScript/Node.js SDK** — for OpenAI, Anthropic, and Vercel AI SDK integrations
- [ ] **gRPC server mode** — expose verification as a sidecar service for containerized agent deployments
- [ ] **Prometheus metrics endpoint** — export accept/reject rates, latency histograms, chain depth

---

## V4 — Decentralized Audit

Goal: enable third-party verification and on-chain settlement.

- [ ] **Merkle challenge API** — serve inclusion proofs for individual receipts against a published slab root
- [ ] **On-chain anchor** — publish slab Merkle roots to an EVM-compatible chain for immutable audit trail
- [ ] **Multi-party verification** — allow multiple independent validators to co-sign a slab
- [ ] **Compliance exports** — generate audit reports in SOC 2 / ISO 27001-compatible formats

---

## Research Directions

- **Formal verification of the Rust implementation** — use Kani or Verus to prove memory safety + arithmetic correctness directly on the Rust code, closing the gap between the Lean proof and the binary.
- **ZK receipt compression** — use a SNARK to produce a constant-size proof that a slab satisfies the accounting law, without revealing individual step values.
- **Continuous canon evolution** — formal process for versioning the `canon_profile_hash` without breaking existing receipts.
