import Coh.Prelude

namespace Coh.Ledger

universe u v

/--
Lightweight small-category structure used for the formal ledger branch.
-/
structure SmallCategory (X : Type u) where
  Hom : X ? X ? Type v
  id : (x : X) ? Hom x x
  comp : {x y z : X} ? Hom y z ? Hom x y ? Hom x z
  comp_assoc :
    ? {w x y z : X} (h : Hom y z) (g : Hom x y) (f : Hom w x),
      comp h (comp g f) = comp (comp h g) f
  id_comp : ? {x y : X} (f : Hom x y), comp (id y) f = f
  comp_id : ? {x y : X} (f : Hom x y), comp f (id x) = f

/--
Formal data of a strict Coh system.
-/
structure StrictCohSystem (X : Type u) (M : Type v) where
  source : M ? X
  target : M ? X
  comp : M ? M ? M
  id : X ? M
  RV : M ? Bool
  source_comp : ? {f g : M}, target f = source g ? source (comp g f) = source f
  target_comp : ? {f g : M}, target f = source g ? target (comp g f) = target g
  SC1 : ? {f g : M}, RV f = true ? RV g = true ? target f = source g ? RV (comp g f) = true
  SC2 : ? x : X, RV (id x) = true
  SC3 : ? {f g h : M}, target f = source g ? target g = source h ? comp h (comp g f) = comp (comp h g) f
  SC4_right : ? f : M, comp f (id (source f)) = f
  SC4_left : ? f : M, comp (id (target f)) f = f
  source_id : ? x : X, source (id x) = x
  target_id : ? x : X, target (id x) = x

namespace StrictCohSystem

/-- Admissibility predicate cut out by the deterministic verifier. -/
def Admissible {X : Type u} {M : Type v} (S : StrictCohSystem X M) (f : M) : Prop :=
  S.RV f = true

/-- Admissible arrows with prescribed endpoints. -/
def Hom {X : Type u} {M : Type v} (S : StrictCohSystem X M) (x y : X) : Type v :=
  {f : M // S.Admissible f ? S.source f = x ? S.target f = y}

/-- Identity arrow in the admissible fragment. -/
def idHom {X : Type u} {M : Type v} (S : StrictCohSystem X M) (x : X) : S.Hom x x :=
  ?S.id x, S.SC2 x, S.source_id x, S.target_id x?

/-- Composition in the admissible fragment. -/
def compHom {X : Type u} {M : Type v} (S : StrictCohSystem X M)
    {x y z : X} (g : S.Hom y z) (f : S.Hom x y) : S.Hom x z := by
  rcases f with ?f, hfAdm, hfSrc, hfTgt?
  rcases g with ?g, hgAdm, hgSrc, hgTgt?
  refine ?S.comp g f, ?_?
  have hcomp : S.target f = S.source g := by
    calc
      S.target f = x := hfTgt
      _ = S.source g := hgSrc.symm
  refine ?S.SC1 hfAdm hgAdm hcomp, ?_, ?_?
  · exact (S.source_comp hcomp).trans hfSrc
  · exact (S.target_comp hcomp).trans hgTgt

/-- Step-by-step verification of T1 Category Axioms. -/
def admissibleFragment {X : Type u} {M : Type v} (S : StrictCohSystem X M) : SmallCategory X where
  Hom := S.Hom
  id := S.idHom
  comp := fun {x y z} g f => S.compHom g f
  comp_assoc := by
    intro w x y z h g f
    rcases f with ?f, hfAdm, hfSrc, hfTgt?
    rcases g with ?g, hgAdm, hgSrc, hgTgt?
    rcases h with ?h, hhAdm, hhSrc, hhTgt?
    apply Subtype.ext
    have hfg : S.target f = S.source g := by
      calc
        S.target f = x := hfTgt
        _ = S.source g := hgSrc.symm
    have hgh : S.target g = S.source h := by
      calc
        S.target g = y := hgTgt
        _ = S.source h := hhSrc.symm
    exact S.SC3 hfg hgh
  id_comp := by
    intro x y f
    rcases f with ?f, hfAdm, hfSrc, hfTgt?
    apply Subtype.ext
    simpa [idHom, compHom, hfTgt] using S.SC4_left f
  comp_id := by
    intro x y f
    rcases f with ?f, hfAdm, hfSrc, hfTgt?
    apply Subtype.ext
    simpa [idHom, compHom, hfSrc] using S.SC4_right f

/-- T1 Theorem Conclusion: $\mathcal C_{\mathrm{adm}}(S)$ is a category. -/
theorem T1_StrictCoh_to_Category {X : Type u} {M : Type v} (S : StrictCohSystem X M) :
    Nonempty (SmallCategory X) :=
  ?S.admissibleFragment?

end StrictCohSystem

end Coh.Ledger
