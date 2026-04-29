# Coh-Wedge Remediation Roadmap

Multi-phase plan addressing five systemic weak points.

## Overview

| Phase | Focus Area | Duration |
|-------|-----------|----------|
| **Phase 1** | Foundation - Entropy floors + Lake integration | 2-3 sprints | **✓ COMPLETE** |

**Phase 1 Status:**
- ✅ Entropy floors added to [`phaseloom_lite.rs`](coh-node/crates/coh-genesis/src/phaseloom_lite.rs)
- ✅ Temperature schedule (Boltzmann exploration)
- ✅ 10 tests passing
- ✅ Lake environment interface added to [`mathlib_advisor.rs`](coh-node/crates/coh-genesis/src/mathlib_advisor.rs)
- ✅ Heuristics fallback when lake unavailable
| **Phase 2** | Lean Proof Closure | 3-4 sprints | **✓ COMPLETE** |

**Phase 2 Status:**
- ✅ Proof debt cataloged - no `sorry` in Lean files
- ✅ [`RationalInf.lean`](coh-t-stack/Coh/Boundary/RationalInf.lean): Full proofs
- ✅ [`Formation.lean`](coh-t-stack/Coh/Boundary/Formation.lean): Full definitions  
- ✅ [`LawOfGenesis.lean`](coh-t-stack/Coh/Boundary/LawOfGenesis.lean): Full definitions
| **Phase 3** | BKM Positioning | 1-2 sprints | **✓ COMPLETE** |

**Phase 3 Status:**
- ✅ Conditional scope documented in [`DyadicBKMBridge.lean`](coh-t-stack/Coh/Boundary/DyadicBKMBridge.lean:15)
- ✅ Added `Remark` explaining conditional approximation
- ✅ Added `STATUS` and `NOT_GLOBAL_REGULARITY` flags
| Phase 4 | Integration - Real proofs in generator pipeline | 2-3 sprints |

---

## Phase 1: Foundation

### 1.1 Add Entropy/Exploration Floors to PhaseLoom

**Problem**: Strategy weights overfit because [`phaseloom_lite.rs`](coh-node/crates/coh-genesis/src/phaseloom_lite.rs:191) samples by "highest weight" only.

**Required Changes**:

1. In [`phaseloom_lite.rs`](coh-node/crates/coh-genesis/src/phaseloom_lite.rs:51):
   - Add `entropy_floor` config parameter (e.g., 0.1)
   - Add Boltzmann exploration: `P(c) ∝ exp(weight[c] / temperature)`
   - Add `temperature` schedule (decay over iterations)

2. Replace deterministic `sample_strategy()` with probabilistic sampling:
   ```rust
   pub fn sample_strategy(&self, temperature: f64) -> Option<String> {
       // Boltzmann distribution over weights
       let weights = &self.strategy_weights.0;
       if weights.is_empty() { return None; }
       
       let sum: f64 = weights.values()
           .map(|w| (w / temperature).exp())
           .sum();
       
       // Sample proportional to exp(w/T)
       // ... 
   }
   ```

3. Add circuit-breaker for exploration exhaustion:
   - Track consecutive exploration failures
   - Force exploitation after threshold

**Deliverable**: `phaseloom_lite.rs` with entropy floor and temperature schedule.

### 1.2 Mathlib Advisor → Lake Environment Integration

**Problem**: [`mathlib_advisor.rs`](coh-node/crates/coh-genesis/src/mathlib_advisor.rs:31) uses heuristic strategy enums, not actual Lean search.

**Required Changes**:

1. Add lake environment query wrapper:
   ```rust
   pub struct MathlibLakeQuery {
       // Path to lean project
       pub project_path: PathBuf,
   }
   
   impl MathlibLakeQuery {
       /// Query mathlib for lemmas matching term patterns
       pub fn search(&self, query: &str) -> Result<Vec<LemmaMatch>> {
           // Run `lake exe lean --irthm` or similar
       }
       
       /// Get available instances for type class
       pub fn instances(&self, class: &str) -> Result<Vec<String>> {
           // Query `#check` for instances
       }
   }
   ```

2. Update [`mathlib_advisor.rs`](coh-node/crates/coh-genesis/src/mathlib_advisor.rs:12) to:
   - Accept optional `lake_env` for authoritative answers
   - Fall back to heuristic strategies when lake unavailable
   - Track and report lake query latency

3. Add verification report field:
   ```rust
   pub struct LeanVerificationReport {
       // ... existing fields
       pub lemmas_validated: Vec<String>,  // From lake
       pub instances_verified: Vec<String>,
   }
   ```

**Deliverable**: `mathlib_advisor.rs` with optional lake integration, heuristics fallback.

---

## Phase 2: Lean Proof Closure

### 2.1 Catalog Current Proof Debt

**Required Audit**:

1. List all `sorry` / `admit` in [`coh-t-stack/`](coh-t-stack/):
   ```bash
   grep -rn "sorry\|admit" coh-t-stack/Coh/
   ```

2. Classify by difficulty:
   - **Trivial** (< 5 tactic repair): `isGLB_of_greatest_of_lower`
   - **Medium** (5-20 tactics): `isRationalInf_pairwise_add`
   - **Hard** (> 20 tactics, requires new lemmas): `dyadic_bkm_bridge`

3. Document scaffolded proofs in `docs/PROOF_DEBT Catalog.md`:

### 2.2 Replace Sorrys with Actual Proofs

**Priority Order**:

1. **Trivial repairs** (e.g., RationalInf.lean):
   - Fix `not_lt_of_le` argument polarity
   - Use explicit structure field access
   - Add missing `NeZero` instances via mathlib

2. **Medium proofs** (e.g., `isRationalInf_pairwise_add`):
   - Reconstruct epsilon-delta argument
   - Import required order theory lemmas
   - Add helper lemmas for glb/inf operations

3. **Bridge theorems** (e.g., DyadicBKMBridge):
   - Connect to existing RationalInf proofs
   - Add approximation chain lemmas
   - Document conditional assumptions

**Process**:
```
1. Run lake build to find failure
2. Use #explore for relevant theorems
3. Construct tactic chain
4. Verify passes
5. Update npe_lean_tactic_repair_v0_3.rs status
```

### 2.3 Remove Proof Scaffolding

**After closure**, remove temporary scaffolding:
- Remove `haveCoe` abstractions added for workaround
- Clean up redundant helper lemmas
- Optimize import lists

---

## Phase 3: BKM Positioning

### 3.1 Document Conditional Scope

**Current state**: [`DyadicBKMBridge.lean`](coh-t-stack/Coh/Boundary/DyadicBKMBridge.lean:15) states:

```lean
theorem dyadic_bkm_bridge {s : Set ENNRat} {i : ENNRat}
  (h_dyadic : ∀ (x : ENNRat), x ∈ s → ∃ (q : NNRat), q = x ∧ IsDyadic q)
  (h_inf : IsRationalInf s i) :
  IsRationalInf s i := h_inf
```

**Issue**: This is essentially a trivial identity, not a meaningful BKM connection.

**Required documentation**:
1. Add preamble explicitly stating conditional scope:
   ```lean
   /--
   Remark: This bridge theorem is conditional on the existence
   of dyadic approximations for all elements. The full BKM
   regularity argument requires additional approximation
   schemes beyond the dyadic case.
   
   Status: CONDITIONAL_APPROXIMATION  -/
   ```

2. Create `docs/BKM_SCOPE.md` explicitly listing:
   - What the bridge assumes
   - What it does NOT claim (global regularity)
   - Required extensions for full BKM

### 3.2 Naming/Audit Trail

Add audit flags to `DyadicBKMBridge.lean`:
```lean
/--
BKM_SCOPE: CONDITIONAL
REQUIRES: dyadic approximation existence
NOT_GLOBAL_REGULARITY: true
STATUS: placeholder_theorem
-/
```

---

## Phase 4: Generator Pipeline Integration

### 4.1 Real Proof Stream

**Current**: [`generator.rs`](coh-node/crates/coh-genesis/src/generator.rs:1) generates synthetic candidates via RNG.

**Required**: Replace with actual Lean proof generation:

1. Add LLM interface in `coh-genesis`:
   ```rust
   pub struct NpeLeanInterface {
       pub llm_endpoint: String,
       pub model: String,
   }
   
   impl NpeLeanInterface {
       /// Generate proof candidates for theorem
       pub fn generate_proofs(&self, theorem: &str) 
           -> Vec<ProofCandidate> {
           // Call LLM with theorem + context
           // Parse Lean proof text
       }
   }
   ```

2. Integrate with PhaseLoom:
   - Pass strategy weights as LLM prompt bias
   - Include entropy-adjusted exploration prompts

3. Add verification loop:
   ```rust
   pub fn verify_and_adapt(
       candidates: Vec<ProofCandidate>
   ) -> Receipt {
       for candidate in candidates {
           let report = lean_verify(&candidate);
           if report.compiles && !report.has_sorry {
               return Receipt { accepted: true, ... };
           }
       }
       // All rejected - update PhaseLoom curvature
   }
   ```

### 4.2 Benchmark Distinction

**Required metrics**:
- Track `proof_origin`: synthetic | llm | hybrid
- Separate benchmark reporting:
  ```rust
  pub struct BenchmarkReport {
      pub synthetic_proven: u32,
      pub llm_proven: u32,
      pub benchmark_only: u32,  // Not actual Lean proofs
  }
  ```

### 4.3 Proof Debt Tracking

Add dashboard metrics:
- `# sorry` count (declining)
- `avg_tactic_depth` per theorem
- `proof_latency_ms` for LLM generation

---

## Risk Register

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| LLM integration timeout | Medium | Configurable timeout, fallback to synthetic |
| Mathlib version drift | Medium | Pin mathlib version in lake-manifest.json |
| PhaseLoom entropy causes instability | Low | Configurable floor, monitor convergence |
| BKM scope rejected by reviewers | Medium | Explicit conditional framing from start |

---

## Success Criteria

| Phase | Criteria |
|-------|----------|
| Phase 1 | Entropy floor active, lake queries functional |
| Phase 2 | All `sorry` resolved, benchmarks labeled |
| Phase 3 | BKM conditional documented in code |
| Phase 4 | Real proofs > 50% of Genesis admissions |

---

## Dependencies

```
Phase 1 ──┬──> Phase 2
          │       (entropy prevents overfitting during closure)
          │
          └──> Phase 4
                  (lake integration required for LLM verification)

Phase 2 ──> Phase 3
          (proof closure clarifies BKM scope)

Phase 3 ──> Phase 4
          (conditional BKM enables honest pipeline)
```

---

## Notes

- **NOT claiming global regularity**: BKM remains conditional throughout
- **Benchmark labels**: Must distinguish synthetic/benchmark from real proofs in all reporting
- **Entropy floors**: Stay active forever - no overfit even in production