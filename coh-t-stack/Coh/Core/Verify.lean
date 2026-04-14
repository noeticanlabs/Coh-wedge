import Coh.Core.Law
import Mathlib.Tactic.Linarith

namespace Coh.Core

open Coh.Kernel

abbrev Decision := Coh.Kernel.Decision
abbrev verify := Coh.Kernel.verify

theorem verify_accept_iff_lawful (r : Receipt) :
    verify r = Decision.accept ↔ Lawful r :=
  Coh.Kernel.verify_accept_iff r

theorem accepted_step_obeys_law_of_coherence (r : Receipt) :
    Accepted r → Lawful r := by
  intro hAccepted
  exact (verify_accept_iff_lawful r).mp hAccepted

theorem accepted_implies_nonnegative_descent (r : Receipt)
    (hAccepted : Accepted r) (hBudget : r.defect ≤ r.spend) :
    r.post ≤ r.pre + r.authority := by
  have hLawful : Lawful r := accepted_step_obeys_law_of_coherence r hAccepted
  unfold Lawful Coh.Kernel.Lawful at hLawful
  linarith

end Coh.Core
