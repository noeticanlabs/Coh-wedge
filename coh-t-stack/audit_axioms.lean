import Coh

/-! Axiom audit surface.

Running `#print axioms` on every load-bearing theorem lets CI mechanically
detect any axiom regression (e.g., accidental `sorry` or new `axiom`).

Expected axioms for the core chain: `propext`, `Quot.sound`, `Classical.choice`.
Expected axiom for T5: `Coh.Selection.clifford_algebra_dimension`.
-/

-- Core kernel
#print axioms Coh.Kernel.verify_accept_iff
#print axioms Coh.Core.verify_accept_iff_lawful

-- Contract verifier
#print axioms Coh.Contract.rv_contract_correctness

-- Trace
#print axioms Coh.Core.accepted_trace_closure
#print axioms Coh.Core.accepted_step_implies_chain_digest_correct
#print axioms Coh.Core.acceptedTrace_endState_unique

-- Oplax algebra
#print axioms Coh.Oplax.oplax_comp_assoc

-- Crypto refinement
#print axioms Coh.Crypto.digestUpdate_refines_sha256_spec
#print axioms Coh.Crypto.compute_chain_digest_eq_spec

-- T1 persistence
#print axioms Coh.Kernel.t1_nontrivial_propagation

-- T5 Dirac (should show clifford_algebra_dimension)
#print axioms Coh.Selection.T5_Dirac_inevitability

-- T3 grounding
#print axioms Coh.Trace.t3_accepted_trace_implies_slab_verified

-- T4 grounding
#print axioms Coh.Spectral.receipt_defect_implies_visibility
