import Coh.Kernel.Receipt

namespace Coh.Kernel

/-- The Accounting Law: 
    1. v_post + spend <= v_pre + defect + authority (Conservation)
    2. spend <= v_pre (Domain Constraint)
-/
def Lawful (r : Receipt) : Prop :=
  r.post + r.spend <= r.pre + r.defect + r.authority ∧ r.spend <= r.pre

/-- Runtime Rejection Codes. -/
inductive Decision
  | accept
  | reject
  deriving DecidableEq

/-- The Operational Verifier logic. -/
noncomputable def verify (r : Receipt) : Decision :=
  if r.post + r.spend <= r.pre + r.defect + r.authority ∧ r.spend <= r.pre then Decision.accept else Decision.reject

/-- Theorem A.2.1: verify_accept_iff -/
lemma verify_accept_iff (r : Receipt) : verify r = Decision.accept ↔ Lawful r := by
  unfold verify Lawful
  split <;> simp_all

/-- Theorem A.2.2: verify_sound -/
lemma verify_sound (r : Receipt) : verify r = Decision.accept → Lawful r :=
  (verify_accept_iff r).mp

/-- Theorem A.2.3: verify_complete -/
lemma verify_complete (r : Receipt) : Lawful r → verify r = Decision.accept :=
  (verify_accept_iff r).mpr

end Coh.Kernel
