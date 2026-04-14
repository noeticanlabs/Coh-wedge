import Coh.Core.Law

namespace Coh.Oplax

open Coh.Core

/-- Oplax morphisms preserve lawfulness up to an additive slack budget. -/
structure OplaxMorphism where
  map : Receipt → Receipt
  slack : ℝ
  preserves :
    ∀ {Δ : ℝ} {r : Receipt}, LawfulUpTo Δ r → LawfulUpTo (Δ + slack) (map r)

/-- Composition of oplax morphisms adds slack. -/
def comp (G F : OplaxMorphism) : OplaxMorphism where
  map r := G.map (F.map r)
  slack := F.slack + G.slack
  preserves := by
    intro Δ r hLawful
    have hF : LawfulUpTo (Δ + F.slack) (F.map r) := F.preserves hLawful
    have hG : LawfulUpTo ((Δ + F.slack) + G.slack) (G.map (F.map r)) := G.preserves hF
    simpa [add_assoc, add_left_comm, add_comm] using hG

theorem oplax_compose_slack_add (G F : OplaxMorphism) :
    (comp G F).slack = F.slack + G.slack := rfl

end Coh.Oplax
