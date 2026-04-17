import Mathlib.CategoryTheory.Category.Basic

/-!
# Coh.Category.GovCat

Robust categorical core for governed systems with deterministic verifier
semantics and optional accounting structure. This module provides:

- Decision (accept | reject code)
- GovObj σ: objects with state, receipt, code, canon profile, potential V,
  Spend/Defect, and a deterministic verifier RV with a soundness law
- Strict morphisms Hom preserving accepted steps
- Category instance `Category (GovObj σ)` (mathlib Category)
- OplaxHom with explicit nonnegative slack Δ and composition law (Δ-additive)

Notes
- We parametrize on a global scalar σ with ordered additive structure, which
  allows stating ledger-like inequalities uniformly across objects.
- The oplax layer requires nonnegativity of Δ to make weakening monotone.
- Engineering-facing receipt schemas and boundary verifiers should live in a
  separate contract layer and be bridged to GovObj via soundness theorems.
-/

universe u v w

namespace Coh

/-- Deterministic verifier decision. -/
inductive Decision (Code : Type u) : Type u
| accept : Decision Code
| reject : Code → Decision Code
deriving DecidableEq, Repr

/-- Minimal canon profile placeholder (kept abstract in the core). -/
structure CanonProfile : Type (u + 1) where
  profileId : String
deriving Repr, DecidableEq

variable (σ : Type) [OrderedAddCommMonoid σ]

/-- Governed object with accounting structure and a deterministic verifier. -/
structure GovObj : Type (max (u+1) (v+1) (w+1)) where
  X       : Type u
  Receipt : Type v
  Code    : Type w
  canon   : CanonProfile
  V       : X → σ
  Spend   : Receipt → σ
  Defect  : Receipt → σ
  RV      : X → Receipt → X → Decision Code
  /-- Verifier soundness law: accepted steps satisfy the ledger inequality. -/
  rv_sound : ∀ {x : X} {r : Receipt} {x' : X},
    RV x r x' = Decision.accept →
    V x' + Spend r ≤ V x + Defect r

namespace GovObj

variable {σ} [OrderedAddCommMonoid σ]

@[simp] def Legal (S : GovObj σ) (x : S.X) (r : S.Receipt) (x' : S.X) : Prop :=
  S.RV x r x' = Decision.accept

end GovObj

/-- Strict legality-preserving morphisms. -/
structure Hom {σ : Type} [OrderedAddCommMonoid σ]
  (S T : GovObj σ) : Type (max u v w) where
  fX : S.X → T.X
  fR : S.Receipt → T.Receipt
  sound : ∀ {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV x r x' = Decision.accept →
    T.RV (fX x) (fR r) (fX x') = Decision.accept

namespace Hom

variable {σ : Type} [OrderedAddCommMonoid σ]

@[simp] def id (S : GovObj σ) : Hom S S :=
  { fX := id, fR := id, sound := by intro x r x' h; simpa using h }

@[simp] def comp {S T U : GovObj σ} (f : Hom S T) (g : Hom T U) : Hom S U :=
  { fX := g.fX ∘ f.fX
  , fR := g.fR ∘ f.fR
  , sound := by
      intro x r x' h
      have hT : T.RV (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.sound h
      exact g.sound hT }

end Hom

instance {σ : Type} [OrderedAddCommMonoid σ] : Category (GovObj σ) where
  Hom S T := Hom S T
  id := Hom.id
  comp := fun g f => Hom.comp f g
  id_comp := by intro S T f; cases f; rfl
  comp_id := by intro S T f; cases f; rfl
  assoc := by intro A B C D f g h; cases f; cases g; cases h; rfl

/-- Oplax morphisms: legality preservation plus two ledger laws with explicit slack. -/
structure OplaxHom {σ : Type} [OrderedAddCommMonoid σ]
  (S T : GovObj σ) : Type (max u v w) where
  fX : S.X → T.X
  fR : S.Receipt → T.Receipt
  /-- Strict part: accepted steps in S are accepted after transport to T. -/
  strict : ∀ {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV x r x' = Decision.accept →
    T.RV (fX x) (fR r) (fX x') = Decision.accept
  /-- Nonnegative slack. -/
  Δ  : σ
  Δ_nonneg : 0 ≤ Δ
  /-- Target-only ledger inequality for transported steps (within T). -/
  ledger : ∀ {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV x r x' = Decision.accept →
    T.V (fX x') + T.Spend (fR r) ≤ T.V (fX x) + T.Defect (fR r) + Δ
  /-- Source→Target bridge on (pre, defect): relates T back to S. -/
  bridge : ∀ {x : S.X} {r : S.Receipt} {x' : S.X},
    S.RV x r x' = Decision.accept →
    T.V (fX x) + T.Defect (fR r) ≤ S.V x + S.Defect r + Δ

namespace OplaxHom

variable {σ : Type} [OrderedAddCommMonoid σ]

@[simp] def id (S : GovObj σ) : OplaxHom S S :=
  { fX := id
  , fR := id
  , strict := by intro x r x' h; simpa using h
  , Δ := 0
  , Δ_nonneg := by exact le_rfl
  , ledger := by
      intro x r x' h
      -- From object soundness at S with identity transport, inequality holds with Δ=0
      simpa [add_zero] using (S.rv_sound h)
  , bridge := by
      intro x r x' h
      -- Trivial bound: V x + Defect r ≤ V x + Defect r + 0
      have : S.V x + S.Defect r ≤ S.V x + S.Defect r := le_rfl
      simpa [add_zero] using this }

@[simp] def comp {S T U : GovObj σ}
  (f : OplaxHom S T) (g : OplaxHom T U) : OplaxHom S U :=
  { fX := g.fX ∘ f.fX
  , fR := g.fR ∘ f.fR
  , strict := by
      intro x r x' h
      have hT := f.strict h
      exact g.strict hT
  , Δ := f.Δ + g.Δ
  , Δ_nonneg := by
      have := add_nonneg f.Δ_nonneg g.Δ_nonneg
      simpa using this
  , ledger := by
      intro x r x' hS
      have hT : T.RV (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.strict hS
      have hg : U.V (g.fX (f.fX x')) + U.Spend (g.fR (f.fR r)) ≤
                 U.V (g.fX (f.fX x))  + U.Defect (g.fR (f.fR r)) + g.Δ :=
        g.ledger hT
      have hweak :
        U.V (g.fX (f.fX x')) + U.Spend (g.fR (f.fR r)) ≤
        U.V (g.fX (f.fX x))  + U.Defect (g.fR (f.fR r)) + g.Δ + f.Δ := by
        have : (U.V (g.fX (f.fX x')) + U.Spend (g.fR (f.fR r))) + 0 ≤
               (U.V (g.fX (f.fX x))  + U.Defect (g.fR (f.fR r)) + g.Δ) + f.Δ := by
          exact add_le_add hg f.Δ_nonneg
        simpa [add_comm, add_left_comm, add_assoc, add_zero] using this
      simpa [add_comm, add_left_comm, add_assoc] using hweak
  , bridge := by
      intro x r x' hS
      -- Bridge via g from T to U (after acceptance in T)
      have hT : T.RV (f.fX x) (f.fR r) (f.fX x') = Decision.accept := f.strict hS
      have gb : U.V (g.fX (f.fX x)) + U.Defect (g.fR (f.fR r)) ≤
                 T.V (f.fX x)        + T.Defect (f.fR r)        + g.Δ :=
        g.bridge hT
      -- Bridge via f from S to T on (pre,defect)
      have fb : T.V (f.fX x) + T.Defect (f.fR r) ≤
                 S.V x        + S.Defect r        + f.Δ :=
        f.bridge hS
      -- Compose the bounds and reassociate to total slack f.Δ + g.Δ
      have : U.V (g.fX (f.fX x)) + U.Defect (g.fR (f.fR r)) ≤
             S.V x + S.Defect r + g.Δ + f.Δ := by
        have := add_le_add_right fb g.Δ
        exact le_trans gb (by simpa [add_comm, add_left_comm, add_assoc] using this)
      simpa [add_comm, add_left_comm, add_assoc] using this }

end OplaxHom

/-!
## Pos-enrichment: preorder on OplaxHom by slack tightness

We order oplax morphisms by their slack: f ≤ g :↔ f.Δ ≤ g.Δ.
This equips each hom-set with a canonical preorder reflecting tightness.
-/

namespace OplaxHom

variable {σ : Type} [OrderedAddCommMonoid σ]

@[simp] def tightLe {S T : GovObj σ} (f g : OplaxHom S T) : Prop := f.Δ ≤ g.Δ

instance instPreorder {S T : GovObj σ} : Preorder (OplaxHom S T) where
  le := tightLe
  lt f g := tightLe f g ∧ ¬ tightLe g f
  le_refl f := le_rfl
  le_trans f g h hfg hgh := le_trans hfg hgh

end OplaxHom

end Coh
