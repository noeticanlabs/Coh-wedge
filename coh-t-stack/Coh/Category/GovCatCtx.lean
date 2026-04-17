import Mathlib.CategoryTheory.Category.Basic
import Coh.Category.GovCat
import Coh.Contract.Boundary

/-!
# Coh.Category.GovCatCtx

Context-threaded governed category. This generalizes `GovCat` by allowing the
verifier to depend on an explicit step context (e.g., previous chain digest).

We preserve the strict and oplax layers with Δ-additive composition. An adapter
`fromGovObj` embeds any context-free `GovObj` into this contextful category by
ignoring the context parameter.
-/

universe u v w

namespace Coh

/-- Minimal step context, carrying prior chain-digest bytes. -/
structure StepCtx where
  prevChainDigest : Coh.Contract.Digest
deriving Repr, DecidableEq

variable (σ : Type) [OrderedAddCommMonoid σ]

/-- Governed object with explicit step context. -/
structure GovObjCtx : Type (max (u+1) (v+1) (w+1)) where
  X       : Type u
  Receipt : Type v
  Code    : Type w
  canon   : CanonProfile
  V       : X → σ
  Spend   : Receipt → σ
  Defect  : Receipt → σ
  RV      : StepCtx → X → Receipt → X → Decision Code
  rv_sound : ∀ {c : StepCtx} {x : X} {r : Receipt} {x' : X},
    RV c x r x' = Decision.accept →
    V x' + Spend r ≤ V x + Defect r

namespace GovObjCtx

variable {σ} [OrderedAddCommMonoid σ]

@[simp] def Legal (S : GovObjCtx σ) (c : StepCtx) (x : S.X) (r : S.Receipt) (x' : S.X) : Prop :=
  S.RV c x r x' = Decision.accept

end GovObjCtx

/-- Strict context-preserving morphisms. -/
structure Hom {σ : Type} [OrderedAddCommMonoid σ]
  (S T : GovObjCtx σ) : Type (max u v w) where
  fX : S.X → T.X
  fR : S.Receipt → T.Receipt
  sound : ∀ {c : StepCtx} {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV c x r x' = Decision.accept →
    T.RV c (fX x) (fR r) (fX x') = Decision.accept

namespace Hom

variable {σ : Type} [OrderedAddCommMonoid σ]

@[simp] def id (S : GovObjCtx σ) : Hom S S :=
  { fX := id, fR := id, sound := by intro c x r x' h; simpa using h }

@[simp] def comp {S T U : GovObjCtx σ} (f : Hom S T) (g : Hom T U) : Hom S U :=
  { fX := g.fX ∘ f.fX
  , fR := g.fR ∘ f.fR
  , sound := by
      intro c x r x' h
      have hT : T.RV c (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.sound h
      exact g.sound hT }

end Hom

instance {σ : Type} [OrderedAddCommMonoid σ] : Category (GovObjCtx σ) where
  Hom S T := Hom S T
  id := Hom.id
  comp := fun g f => Hom.comp f g
  id_comp := by intro S T f; cases f; rfl
  comp_id := by intro S T f; cases f; rfl
  assoc := by intro A B C D f g h; cases f; cases g; cases h; rfl

/-- Oplax morphisms with explicit context and nonnegative slack Δ. -/
structure OplaxHom {σ : Type} [OrderedAddCommMonoid σ]
  (S T : GovObjCtx σ) : Type (max u v w) where
  fX : S.X → T.X
  fR : S.Receipt → T.Receipt
  strict : ∀ {c : StepCtx} {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV c x r x' = Decision.accept →
    T.RV c (fX x) (fR r) (fX x') = Decision.accept
  Δ  : σ
  Δ_nonneg : 0 ≤ Δ
  ledger : ∀ {c : StepCtx} {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV c x r x' = Decision.accept →
    T.V (fX x') + T.Spend (fR r) ≤ T.V (fX x) + T.Defect (fR r) + Δ
  bridge : ∀ {c : StepCtx} {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV c x r x' = Decision.accept →
    T.V (fX x) + T.Defect (fR r) ≤ S.V x + S.Defect r + Δ

namespace OplaxHom

variable {σ : Type} [OrderedAddCommMonoid σ]

@[simp] def id (S : GovObjCtx σ) : OplaxHom S S :=
  { fX := id
  , fR := id
  , strict := by intro c x r x' h; simpa using h
  , Δ := 0
  , Δ_nonneg := le_rfl
  , ledger := by intro c x r x' h; simpa [add_zero] using (S.rv_sound h)
  , bridge := by intro c x r x' h; simpa [add_zero] using (le_rfl : S.V x + S.Defect r ≤ S.V x + S.Defect r) }

@[simp] def comp {S T U : GovObjCtx σ}
  (f : OplaxHom S T) (g : OplaxHom T U) : OplaxHom S U :=
  { fX := g.fX ∘ f.fX
  , fR := g.fR ∘ f.fR
  , strict := by intro c x r x' h; exact g.strict (f.strict h)
  , Δ := f.Δ + g.Δ
  , Δ_nonneg := by simpa using add_nonneg f.Δ_nonneg g.Δ_nonneg
  , ledger := by
      intro c x r x' hS
      have hT : T.RV c (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.strict hS
      have hg := g.ledger hT
      have : U.V (g.fX (f.fX x')) + U.Spend (g.fR (f.fR r)) ≤
              U.V (g.fX (f.fX x))  + U.Defect (g.fR (f.fR r)) + g.Δ + f.Δ := by
        exact (add_le_add hg f.Δ_nonneg)
      simpa [add_comm, add_left_comm, add_assoc, add_zero] using this
  , bridge := by
      intro c x r x' hS
      have hT : T.RV c (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.strict hS
      have gb : U.V (g.fX (f.fX x)) + U.Defect (g.fR (f.fR r)) ≤
                T.V (f.fX x)        + T.Defect (f.fR r)        + g.Δ := g.bridge hT
      have fb : T.V (f.fX x) + T.Defect (f.fR r) ≤ S.V x + S.Defect r + f.Δ := f.bridge hS
      have : U.V (g.fX (f.fX x)) + U.Defect (g.fR (f.fR r)) ≤ S.V x + S.Defect r + g.Δ + f.Δ := by
        have := add_le_add_right fb g.Δ
        exact le_trans gb (by simpa [add_comm, add_left_comm, add_assoc] using this)
      simpa [add_comm, add_left_comm, add_assoc] using this }

end OplaxHom

/-- Tightness preorder for OplaxHom: f ≤ g when f.Δ ≤ g.Δ.
    This enables comparing slack values between oplax morphisms. -/
instance OplaxHom_preorder (S T : GovObjCtx σ) : Preorder (OplaxHom S T) where
  le f g := f.Δ ≤ g.Δ
  lt f g := f.Δ < g.Δ
  le_refl f := le_rfl
  le_trans f g h := le_trans
  lt_trans f g h := lt_trans
  lt_of_le_of_lt f g h := lt_of_le_of_lt
  le_of_lt_of_le f g h := le_of_lt_of_le

/-- Adapter: view a context-free governed object as a contextful one by
    ignoring the context parameter. -/
def fromGovObj {σ : Type} [OrderedAddCommMonoid σ] (S : GovObj σ) : GovObjCtx σ :=
  { X := S.X
  , Receipt := S.Receipt
  , Code := S.Code
  , canon := S.canon
  , V := S.V
  , Spend := S.Spend
  , Defect := S.Defect
  , RV := fun _ x r x' => S.RV x r x'
  , rv_sound := by intro c x r x' h; exact S.rv_sound h }

/-- [TESTED]
Example lemma: fromGovObj preserves rv_sound. The soundness law holds in the
contextful category because we ignore the context. -/
lemma fromGovObj_rv_sound {σ : Type} [OrderedAddCommMonoid σ]
    (S : GovObj σ) (c : StepCtx) (x : S.X) (r : S.Receipt) (x' : S.X)
    (h : (fromGovObj S).RV c x r x' = Decision.accept) :
    (fromGovObj S).V x' + (fromGovObj S).Spend r ≤ (fromGovObj S).V x + (fromGovObj S).Defect r := by
  simp [fromGovObj] at h
  exact S.rv_sound h

end Coh
