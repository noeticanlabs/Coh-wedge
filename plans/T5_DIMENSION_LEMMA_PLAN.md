# Plan: T5 Dimension Lemma Proof Completion

## Overview
Complete the proof of `dim Cl(ℂ^n) = 2^n` in `coh-t-stack/Coh/Selection/T5_DiracSelection.lean`, replacing remaining `admit` placeholders with constructive proofs.

## Current Status
- **Location**: `coh-t-stack/Coh/Selection/T5_DiracSelection.lean` (lines 92-175)
- **Theorem**: `T5_Dirac_inevitability`
- **Architecture**: Complete (Universal Lift strategy implemented)
- **Remaining**: 3 `admit` placeholders needing constructive proofs

## Todo List

### 1. Analyze remaining admit placeholders in T5_DiracSelection.lean
**Status**: 🔄 In Progress

**Locations**:
- `hf_sq` (line ~133): Polarization identity proof
- `φ_is_surj` (line ~158): Surjectivity proof
- `dim_CliffordAlgebra` (line ~166): Mathlib dimension theorem reference

---

### 2. Complete polarization identity proof (hf_sq)
**Status**: ⏳ Pending

**Strategy**:
The proof requires showing: `f(v)² = Q(v) • 1` where:
- `f(v) = Σ v_i e_i`
- `Q(v) = -Σ η_i² v_i²`

**Mathematical Steps**:
1. Expand `f(v)²` using bilinearity:
   ```
   f(v)² = (Σ_i v_i e_i)(Σ_j v_j e_j)
         = Σ_i v_i² e_i² + Σ_{i≠j} v_i v_j e_i e_j
   ```

2. Apply anticommutation relation `h_gen`:
   - For `i = j`: `e_i² = -η_i² • 1`
   - For `i ≠ j`: `e_i e_j + e_j e_i = 2η_i δ_ij • 1 = 0` (since η is diagonal)

3. Simplify to:
   ```
   f(v)² = Σ_i v_i²(-η_i²) + 0
         = -Σ_i η_i² v_i²
         = Q(v) • 1
   ```

**Lean Implementation**:
```lean
have hf_sq (v : V) : f v * f v = Q v • (1 : A) := by
  calc
    f v * f v = (∑ i, v i • e i) * (∑ j, v j • e j)
      _ = ∑ i j, (v i * v j) • (e i * e j)
      _ = ∑ i, (v i)^2 • (e i * e i) + ∑ i j, (v i * v j) • (e i * e j)
  · rw [h_expand]
  -- Simplify using h_gen
  sorry
```

---

### 3. Complete surjectivity proof (φ_is_surj)
**Status**: ⏳ Pending

**Goal**: Show `Function.Surjective (CliffordAlgebra.lift Q f hf_sq)`

**Strategy**:
- Show image contains all generators `e_i`
- Use `h_span` (adjoin span) to show image = A

**Mathlib API**:
```lean
-- CliffordAlgebra.lift is defined in Mathlib.Algebra.CliffordAlgebra.Basic
-- Signature: lift {R : Type} [CommRing R] {M : Type} [AddCommGroup M]
--   [Module R M] (Q : QuadraticForm R M) (f : M →ₐ[R] A) (h : ∀ m, f m * f m = Q m • 1) :
--   CliffordAlgebra Q →ₐ[R] A
```

**Lean Implementation**:
```lean
have φ_is_surj : Function.Surjective (CliffordAlgebra.lift Q f hf_sq) := by
  -- Show generators e_i are in image
  intro a
  -- Use h_span: Algebra.adjoin ℂ (Set.range e) = ⊤
  sorry
```

---

### 4. Find and reference Mathlib dimension theorem
**Status**: ⏳ Pending

**Requirement**: Prove `Module.dim ℂ (CliffordAlgebra Q) = 2^n`

**Search Strategy**:
1. Search Mathlib for `CliffordAlgebra.dim` or `CliffordAlgebra.finrank`
2. Check `Mathlib.Algebra.CliffordAlgebra.Basic`
3. Fallback: Use `Module.dim` + `FiniteDimensional` instances

**Potential Theorem** (needs verification):
```lean
-- In Mathlib.Algebra.CliffordAlgebra.Basic or .Dimension
theorem CliffordAlgebra.dim_of_nondegenerate
  {R : Type} [Field R] [CharZero R]
  {M : Type} [AddCommGroup M] [Module R M] [FiniteDimensional R M]
  (Q : QuadraticForm R M) (h : Q.IsNondegenerate) :
  Module.dim R (CliffordAlgebra Q) = 2 ^ (Module.dim R M)
```

**Alternative**: Direct computation using PBW basis

---

### 5. Verify complete proof compiles in Lean 4
**Status**: ⏳ Pending

**Steps**:
1. Replace all `admit` with constructive proofs
2. Run `lake build` in `coh-t-stack/`
3. Fix any resulting elaboration errors
4. Verify `T5_Dirac_inevitability` compiles successfully

---

## Detailed Implementation Notes

### Required Imports
```lean
import Mathlib.Algebra.CliffordAlgebra.Basic
import Mathlib.Algebra.QuadraticForm.Basic
import Mathlib.LinearAlgebra.FiniteDimensional
```

### Key Definitions
```lean
-- Vector space V = ℂ^n
let V := Fin n → ℂ

-- Quadratic form Q from η
let Q : QuadraticForm ℂ V := fun v =>
  - ∑ i, (η i)^2 * (v i)^2

-- Linear map f : V → A
let f (v : V) : A := ∑ i, v i • e i
```

### Proof Structure Overview
```
T5_Dirac_inevitability
├── V := Fin n → ℂ
├── Q : QuadraticForm ℂ V
├── f : V → A
├── hf_sq : f(v)² = Q(v) • 1  ← Need constructive proof
├── φ := CliffordAlgebra.lift Q f hf_sq
├── φ_is_surj  ← Need constructive proof
└── dim_CliffordAlgebra = 2^n  ← Need Mathlib theorem
```

---

## Mermaid Diagram: Proof Flow

```mermaid
graph TD
    A[Start: T5_Dirac_inevitability] --> B[Define V = ℂ^n]
    B --> C[Define Q from η]
    C --> D[Define f: V → A]
    D --> E[hf_sq: f(v)² = Q(v)]
    E --> F[φ = CliffordAlgebra.lift Q f hf_sq]
    F --> G[φ_is_surj: Surjective]
    G --> H[dim_Cl(V,Q) = 2ⁿ]
    H --> I[dim A = dim Cl(V,Q) = 2ⁿ]
    E -.->|admit| J[Polarization proof]
    G -.->|admit| K[Surjectivity proof]
    H -.->|admit| L[Mathlib dimension theorem]
```

---

## References
- **Mathlib**: `Mathlib.Algebra.CliffordAlgebra.Basic`
- **Dimension Theorem**: `CliffordAlgebra.finrank` or equivalent
- **Polarization Identity**: Standard algebraic identity
- **PBW Theorem**: Poincaré-Birkhoff-Witt basis for Clifford algebras