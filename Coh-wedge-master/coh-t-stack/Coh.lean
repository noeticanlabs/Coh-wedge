import Coh.Core
import Coh.Contract
import Coh.Crypto
import Coh.Oplax
import Coh.Kernel
import Coh.Slack
import Coh.Trace
import Coh.Spectral
import Coh.Selection
import Coh.Bridges.ContractSoundness
import Coh.Category.GovCatCtx

/-!
Top-level Coh library surface.

Stable public theorem sheet:

### Kernel & Core
- `verify_accept_iff_lawful`
- `accepted_step_obeys_law_of_coherence`
- `accepted_implies_nonnegative_descent`

### Contract Verifier
- `rv_contract_correctness`
- `accepted_step_implies_chain_digest_correct`
- `accepted_step_implies_state_hash_link`

### Trace Determinism
- `accepted_trace_closure`
- `acceptedTrace_endState_unique`
- `acceptedTrace_endState_eq_finalStateHash`

### Oplax Algebra
- `strict_morphism_preserves_lawful`
- `oplax_compose_slack_add`
- `oplax_comp_assoc`
- `strict_embeds_as_zero_slack`
- `zero_slack_oplax_is_strict`

### T1â€“T5 Stack
- `t1_nontrivial_propagation` (T1: persistence witness)
- `t2_hom_is_trivial_subtype` (T2: round-trip faithfulness)
- `t3_accepted_trace_implies_slab_verified` (T3: slab grounding)
- `receipt_defect_implies_visibility` (T4: defect grounding)
- `T5_Dirac_inevitability` (T5: Clifford dimension)

### Bridge Layer
- `MicroBridgeHyp.govObj` (construct governed object from contract verifier)

### Crypto Refinement
- `digestUpdate_refines_sha256_spec`
- `compute_chain_digest_eq_spec`

Axiom footprint: `clifford_algebra_dimension` (cited: Lawsonâ€“Michelsohn 1989,
Atiyahâ€“Bottâ€“Shapiro 1964). All other load-bearing claims are proved.
-/
