# Formal Foundation (Lean 4)

The Coh Safety Wedge relies on a formal foundation written in Lean 4 to prove the core structural properties of the boundary theory. These proofs reside in the `coh-t-stack` directory.

## Core Boundary Laws

The formalization separates the boundary into two interlocking laws, representing the forward generation (Genesis) and backward verification (Coherence).

### 1. The Law of Genesis (Forward Generation)

**Location:** `coh-t-stack/Coh/Boundary/LawOfGenesis.lean`

The Law of Genesis governs the Noetic Proposal Engine (NPE). It defines the criteria for a proposal to be "admissible" for generation.

```lean
def GenesisAdmissible {G P R : Type} [OrderedAddCommMonoid R] 
  (obj : GenesisObject G P R) (g : G) (p : P) (g' : G) : Prop :=
  obj.Gamma g p g' ∧ obj.M g' + obj.C p ≤ obj.M g + obj.D p
```

*   `M`: Mass (or accumulated disorder)
*   `C`: Cost of the proposal
*   `D`: Permitted slack/defect
*   `Gamma`: The structural transition relation

**Key Theorem:** `genesis_composition` proves that the composition of two Genesis-admissible transitions is itself Genesis-admissible, provided costs and slack are additive.

### 2. The Law of Coherence (Backward Verification)

**Location:** `coh-t-stack/Coh/Boundary/LawOfCoherence.lean`

The Law of Coherence governs the Verifier Kernel (RV Kernel). It defines the criteria for a completed execution to be accepted into the canonical timeline.

```lean
def CohAdmissible {X Q S : Type} [OrderedAddCommMonoid S]
  (obj : CoherenceObject X Q S) (x : X) (R : Q) (y : X) : Prop :=
  obj.RV x R y ∧ obj.V y + obj.Spend R ≤ obj.V x + obj.Defect R + obj.Authority R
```

*   `V`: Valuation (safe value or unresolved risk)
*   `Spend`: Resources consumed
*   `Defect`: Allowed variance
*   `Authority`: Systemic override capacity (introduced in V3)
*   `RV`: The structural verification relation

**Key Theorem:** `coherence_composition` proves that the composition of two Coherence-admissible transitions satisfies the additive Law of Coherence. This is the theoretical basis for the `build-slab` aggregation mechanism, guaranteeing that a macro-receipt (slab) is valid if all its micro-receipts are valid.

## Formation

The intersection of these two laws is termed **Formation**. A system that rigorously enforces both the forward generative bound (Genesis) and the backward verification bound (Coherence) operates at the Formation Boundary. This allows for safe, autonomous AI operation where the generative engine explores within mathematically proven structural limits.