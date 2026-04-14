import Coh.Kernel.Verifier
import Mathlib.Tactic.Linarith

namespace Coh.Core

open Coh.Kernel

abbrev Receipt := Coh.Kernel.Receipt

/-- The strict Coh law, re-exported as the stable Pack B semantic predicate. -/
def Lawful (r : Receipt) : Prop :=
  Coh.Kernel.Lawful r

/-- Acceptance in the strict kernel. -/
def Accepted (r : Receipt) : Prop :=
  Coh.Kernel.verify r = Coh.Kernel.Decision.accept

/-- Oplax lawfulness with additive slack. -/
def LawfulUpTo (Δ : ℝ) (r : Receipt) : Prop :=
  r.post + r.spend ≤ r.pre + r.defect + r.authority + Δ

theorem lawful_iff_lawfulUpTo_zero (r : Receipt) :
    Lawful r ↔ LawfulUpTo 0 r := by
  unfold Lawful LawfulUpTo Coh.Kernel.Lawful
  simpa

theorem lawfulUpTo_of_lawful (r : Receipt) {Δ : ℝ}
    (hLawful : Lawful r) (hΔ : 0 ≤ Δ) : LawfulUpTo Δ r := by
  have hZero : LawfulUpTo 0 r := (lawful_iff_lawfulUpTo_zero r).mp hLawful
  unfold LawfulUpTo at *
  linarith

end Coh.Core
