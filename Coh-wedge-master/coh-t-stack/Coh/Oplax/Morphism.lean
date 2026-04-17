import Coh.Oplax.Slack

namespace Coh.Oplax

open Coh.Core

/-- Strict morphisms preserve lawfulness without adding slack. -/
structure StrictMorphism where
  map : Receipt → Receipt
  preserves :
    ∀ {Δ : ℝ} {r : Receipt}, LawfulUpTo Δ r → LawfulUpTo Δ (map r)

theorem strict_morphism_preserves_lawful (F : StrictMorphism) {r : Receipt} :
    Lawful r → Lawful (F.map r) := by
  intro hLawful
  have hZero : LawfulUpTo 0 r := (lawful_iff_lawfulUpTo_zero r).mp hLawful
  have hMap : LawfulUpTo 0 (F.map r) := F.preserves hZero
  exact (lawful_iff_lawfulUpTo_zero (F.map r)).mpr hMap

/-- Strict morphisms embed in the oplax world with zero slack. -/
def toOplax (F : StrictMorphism) : OplaxMorphism where
  map := F.map
  slack := 0
  preserves := by
    intro Δ r hLawful
    simpa using F.preserves hLawful

theorem strict_embeds_as_zero_slack (F : StrictMorphism) {Δ : ℝ} {r : Receipt} :
    LawfulUpTo Δ r →
      LawfulUpTo (Δ + (toOplax F).slack) ((toOplax F).map r) := by
  intro hLawful
  simpa [toOplax] using (toOplax F).preserves (Δ := Δ) (r := r) hLawful

theorem zero_slack_oplax_is_strict (F : OplaxMorphism) (hZero : F.slack = 0)
    {r : Receipt} : Lawful r → Lawful (F.map r) := by
  intro hLawful
  have hZeroLawful : LawfulUpTo 0 r := (lawful_iff_lawfulUpTo_zero r).mp hLawful
  have hMap : LawfulUpTo (0 + F.slack) (F.map r) := F.preserves hZeroLawful
  have hMapZero : LawfulUpTo 0 (F.map r) := by
    simpa [hZero] using hMap
  exact (lawful_iff_lawfulUpTo_zero (F.map r)).mpr hMapZero

end Coh.Oplax
