import Coh.Core.Law

namespace Coh.Oplax

open Coh.Core

/-- Oplax morphisms preserve lawfulness up to an additive slack budget. -/
structure OplaxMorphism where
  map : Receipt â†’ Receipt
  slack : â„
  preserves :
    âˆ€ {Î” : â„} {r : Receipt}, LawfulUpTo Î” r â†’ LawfulUpTo (Î” + slack) (map r)

/-- Composition of oplax morphisms adds slack. -/
def comp (G F : OplaxMorphism) : OplaxMorphism where
  map r := G.map (F.map r)
  slack := F.slack + G.slack
  preserves := by
    intro Î” r hLawful
    have hF : LawfulUpTo (Î” + F.slack) (F.map r) := F.preserves hLawful
    have hG : LawfulUpTo ((Î” + F.slack) + G.slack) (G.map (F.map r)) := G.preserves hF
    simpa [add_assoc, add_left_comm, add_comm] using hG

theorem oplax_compose_slack_add (G F : OplaxMorphism) :
    (comp G F).slack = F.slack + G.slack := rfl

/-- Extensionality for `OplaxMorphism`: two morphisms are equal when their
    `map` functions and `slack` values agree.  The `preserves` field is
    propositional (`Prop`-valued) and closed by proof irrelevance.

    **Proof strategy**: Destructure both records, use `funext` + `subst` to
    unify the data fields (`map`, `slack`), then appeal to `proof_irrel`
    for the remaining `preserves` field. -/
@[ext]
theorem OplaxMorphism.ext
    {F G : OplaxMorphism}
    (hmap   : F.map = G.map)
    (hslack : F.slack = G.slack) : F = G := by
  cases F; cases G
  simp only at hmap hslack
  subst hmap; subst hslack
  rfl

/-- Composition of oplax morphisms is associative.
    Equality follows from `OplaxMorphism.ext`. -/
theorem oplax_comp_assoc (H G F : OplaxMorphism) :
    comp H (comp G F) = comp (comp H G) F := by
  apply OplaxMorphism.ext
  Â· rfl
  Â· simp [comp, add_assoc]

/-- Oplax identity morphism: zero slack, identity map. -/
def id_oplax : OplaxMorphism where
  map r := r
  slack := 0
  preserves := by intro Î” r h; simpa using h

theorem oplax_comp_id (F : OplaxMorphism) :
    comp F id_oplax = F := by
  apply OplaxMorphism.ext
  Â· rfl
  Â· simp [comp, id_oplax]

theorem oplax_id_comp (F : OplaxMorphism) :
    comp id_oplax F = F := by
  apply OplaxMorphism.ext
  Â· rfl
  Â· simp [comp, id_oplax]

end Coh.Oplax
