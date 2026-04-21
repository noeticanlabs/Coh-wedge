# Lean ↔ Rust Traceability Matrix

This document maps the theoretical definitions in the Lean formal stack (`coh-t-stack`) to their runtime implementations in the Rust node (`coh-node`).

## Core Accounting Law

**Lean Definition:**
- [`coh-t-stack/Coh/Kernel/Verifier.lean`](coh-t-stack/Coh/Kernel/Verifier.lean): `Lawful (r : Receipt)`

**Paper Law:** `v_post + spend <= v_pre + defect + authority`

| Lean Concept | Lean Definition Location | Rust Implementation | Rust Location |
|------------|-------------------------|-------------------|------------------|
| `v_post` | `r.post` | `r.metrics.v_post` | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| `v_pre` | `r.pre` | `r.metrics.v_pre` | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| `spend` | `r.spend` | `r.metrics.spend` | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| `defect` | `r.defect` | `r.metrics.defect` | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| `authority` | `r.authority` | `r.metrics.authority` | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| Verifier Logic | `verify` | [`verify_micro()`](coh-node/crates/coh-core/src/verify_micro.rs) | [`coh-node/crates/coh-core/src/verify_micro.rs`](coh-node/crates/coh-core/src/verify_micro.rs) |

## Certified Traces and Composition

**Lean Definition:**
- [`coh-t-stack/Coh/Core/Trace.lean`](coh-t-stack/Coh/Core/Trace.lean): `AcceptedTrace`, `totalSpend`, `totalDefect`

**Paper Law:** Telescoping inequality: `v_post_last + Σ spend <= v_pre_first + Σ defect`

| Lean Concept | Lean Definition Location | Rust Implementation | Rust Location |
|------------|-------------------------|-------------------|------------------|
| `Trace` | `List MicroReceipt` | `Vec<MicroReceipt>` (wire) | [`coh-node/crates/coh-core/src/types.rs`](coh-node/crates/coh-core/src/types.rs) |
| `AcceptedTrace` | `inductive` | [`verify_chain()`](coh-node/crates/coh-core/src/verify_chain.rs) | [`coh-node/crates/coh-core/src/verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) |
| `totalSpend` | `foldl` | cumulative tracking in [`verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) | [`coh-node/crates/coh-core/src/verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) |
| `totalDefect` | `foldl` | cumulative tracking in [`verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) | [`coh-node/crates/coh-core/src/verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) |
| Continuity | `MetricsContinuous` | continuity checks in [`verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) | [`coh-node/crates/coh-core/src/verify_chain.rs`](coh-node/crates/coh-core/src/verify_chain.rs) |
| Trace Determinism | `acceptedTrace_endState_eq_finalStateHash` | deterministic hash computation | [`coh-node/crates/coh-core/src/hash.rs`](coh-node/crates/coh-core/src/hash.rs) |

## Category and Morphisms

**Lean Definition:**
- [`coh-t-stack/Coh/Category/CohDyn.lean`](coh-t-stack/Coh/Category/CohDyn.lean): `DynHom`, `Step`, `path_cost`

| Lean Concept | Lean Definition Location | Rust Implementation | Rust Location |
|------------|-------------------------|-------------------|------------------|
| `DynHom` | `inductive` | [`ExecutionProof`](coh-node/crates/coh-core/src/execute.rs) | [`coh-node/crates/coh-core/src/execute.rs`](coh-node/crates/coh-core/src/execute.rs) |
| `path_cost` | `step_cost` + sum | [`step_cost()`](coh-node/crates/coh-core/src/measurement.rs) | [`coh-node/crates/coh-core/src/measurement.rs`](coh-node/crates/coh-core/src/measurement.rs) |
| `CohDyn` | SmallCategory | Execution loop in [`engine.rs`](coh-node/crates/coh-core/src/trajectory/engine.rs) | [`coh-node/crates/coh-core/src/trajectory/engine.rs`](coh-node/crates/coh-core/src/trajectory/engine.rs) |

## Measurement and Collapse

**Lean Definition:**
- [`coh-t-stack/Coh/Category/Measurement.lean`](coh-t-stack/Coh/Category/Measurement.lean): `Measurement`, `collapses`, `is_oplax`, `Fiber`

| Lean Concept | Lean Definition Location | Rust Implementation | Rust Location |
|------------|-------------------------|-------------------|------------------|
| `Measurement` | CohHom | [`Measurement`](coh-node/crates/coh-core/src/measurement.rs) | [`coh-node/crates/coh-core/src/measurement.rs`](coh-node/crates/coh-core/src/measurement.rs) |
| `collapses` | predicate | [`detect_collapse()`](coh-node/crates/coh-core/src/measurement.rs) | [`coh-node/crates/coh-core/src/measurement.rs`](coh-node/crates/coh-core/src/measurement.rs) |
| `is_oplax` | predicate | [`verify_chain_dissipation()`](coh-node/crates/coh-core/src/measurement.rs) | [`coh-node/crates/coh-core/src/measurement.rs`](coh-node/crates/coh-core/src/measurement.rs) |

## NEW: Semantic Layer (Latest)

**Lean Definition:**
- [`coh-t-stack/Coh/Core/Semantic.lean`](coh-t-stack/Coh/Core/Semantic.lean): `SemanticSystem`, `HiddenTrace`, `Fiber`, `semanticCost`

| Lean Concept | Lean Definition Location | Rust Implementation | Rust Location |
|------------|-------------------------|-------------------|------------------|
| `SemanticSystem` | `structure` | [`SemanticConfig`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |
| `HiddenTrace` | `List H` | [`HiddenTrace`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |
| `project` | `List.map` | [`HiddenTrace::project()`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |
| `Fiber` | `Set` (preimage) | [`RealizableFiber`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |
| `semanticCost` | `sup` over fiber | [`compute_semantic_cost()`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |
| Semantic Subadditivity | `semantic_subadditive` | [`check_semantic_cost_subadditive()`](coh-node/crates/coh-core/src/semantic.rs) | [`coh-node/crates/coh-core/src/semantic.rs`](coh-node/crates/coh-core/src/semantic.rs) |

## Strict-Gap Example

**Lean Example:**
- [`coh-t-stack/Coh/Core/SemanticExample.lean`](coh-t-stack/Coh/Core/SemanticExample.lean): `ToySystem`, `ThetaStar`, `strict_gap`

| Lean Instance | Lean Location | Proposed Rust Vector | Rust Location |
|--------------|-------------|-------------------|--------------|
| `ToySystem` | SemanticExample.lean | N/A (pure Lean proof) | - |
| Strict Gap | `synCost > semCost` | N/A (pure Lean theorem) | - |

**Planned:** Once the Lean example solidifies, create corresponding Rust golden vectors documenting the same gap.
