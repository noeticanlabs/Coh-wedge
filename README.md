Coh Validator

Rust Protocol: Coh V1 Identity: Frozen Wedge

    "Coh Validator is a deterministic CLI tool that verifies state transitions, detects tampering in transition chains, and explains invalid actions with explicit reject codes."

The Coh Validator is the reference "Frozen Wedge" implementation for the Coh protocol. It serves as a high-rigor, deterministic constraint verifier engine that bridge's the formal semantics of the Coh-Lean safety kernel with real-world execution.
?? The Safety Kernel (Core Invariant)

The primary job of the validator is to enforce the Accounting Law of Transitions. For every micro-receipt, the system ensures that:

V p o s t + s p e n d ≤ V p r e + d e f e c t

Where:

    V p r e : Pre-transition potential (metrics).
    V p o s t : Post-transition potential (metrics).
    s p e n d : Consumed potential.
    d e f e c t : Injected potential (usually zero in standard transitions).

Failure to satisfy this inequality results in an immediate RejectRiskBound decision.
??? Command Reference

The validator exposes exactly four commands, designed for use in automated validation pipelines.
1. verify-micro <input.json>

Verifies a single transition receipt in isolation.

    Checks: Schema, Version, Canon Profile, Policy Inequality, and Digest Integrity.
    Input: A single JSON receipt file.

2. verify-chain <input.jsonl>

Verifies a contiguous chain of receipts.

    Checks: All micro-checks, plus state-linkage ( s t a t e h a s h n e x t i = s t a t e h a s h p r e v i + 1 ) and digest-linkage ( c h a i n d i g e s t n e x t i = c h a i n d i g e s t p r e v i + 1 ).
    Input: A JSONL file where each line is a receipt.

3. build-slab <input.jsonl> --out <output.json>

Aggregates a verified chain into a single high-level Slab Receipt.

    Checks: Fully verifies the input chain before aggregation.
    Output: A standalone slab JSON summarizing the ranges and total aggregation.

4. verify-slab <input.json>

Verifies a standalone slab-receipt using macro-accounting logic.

    Checks: Range sanity, micro-count validation, and macro-inequality.

??? Technical Specification
4-Layer Data Model

To eliminate floating-point ambiguity and non-determinism, the validator uses a strict 4-layer architecture:

    Wire Layer: All numerical fields are encoded as Decimal Strings.
    Runtime Layer: Converted to u 128 for exact-integer arithmetic.
    Prehash Layer: Alphabetized canonical view for deterministic hashing.
    Result Layer: Typed decisions (ACCEPT/REJECT) with explicit RejectCode.

Non-Circular Digest Logic

The validator implements a strict non-circular digest rule. The chain_digest_next is computed as:

SHA256("COH_V1_CHAIN" || "|" || chain_digest_prev || "|" || canonical_json(prehash_view))

The prehash view structurally excludes the chain_digest_next field itself, ensuring the digest is a true anchor of the content it receipts.
?? Exit Code Contract

Automation tools can rely on the following exit codes:
Code 	Label 	Description
0 	ACCEPT 	Verification successful.
1 	REJECT 	Semantic rejection (Policy violation, Digest mismatch).
2 	MALFORMED 	Input error (JSON parse error, Invalid HEX, Missing fields).
3 	ERROR 	Internal execution error.
4 	SOURCE 	Invalid source chain provided to build-slab.
?? Getting Started
Installation

cargo build --release -p coh-validator

Running Examples

The examples/ directory contains standard test vectors:

# Valid micro-receipt
coh-validator verify-micro examples/micro_valid.json

# Invalid policy (Risk violation)
coh-validator verify-micro examples/micro_invalid_policy.json

?? Development

    Tests: cargo test -p coh-core runs the internal digest stability vectors.
    Format: Use --format json to get machine-readable output from all commands.

Built with rigor by the Antigravity Team.
