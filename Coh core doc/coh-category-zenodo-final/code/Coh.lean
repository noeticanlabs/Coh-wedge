
inductive GlyphTag
| invoke
| bind
| route
| guard
| emit
deriving DecidableEq, Repr

structure ControlToken where
  opcode : String
  arg    : String
deriving DecidableEq, Repr

structure Glyph where
  tag         : GlyphTag
  surface     : String
  token       : ControlToken
  wf_surface  : Prop
  wf_token    : Prop

def Glyph.compiles (g : Glyph) : Prop :=
  g.wf_surface ∧ g.wf_token

structure Step (X : Type) where
  src         : X
  dst         : X
  glyph       : Glyph
  costSpend   : ℚ
  costDefect  : ℚ
  typed       : Prop
  compiles_ok : glyph.compiles

structure Trace (X : Type) where
  steps : List (Step X)
  chain :
    ∀ i : Nat, i + 1 < steps.length →
      (steps.get ⟨i, Nat.lt_trans (Nat.lt_succ_self i) ‹i + 1 < steps.length›⟩).dst =
      (steps.get ⟨i+1, ‹i + 1 < steps.length›⟩).src

def traceSpend {X : Type} (t : Trace X) : ℚ :=
  t.steps.foldl (fun acc s => acc + s.costSpend) 0

def traceDefect {X : Type} (t : Trace X) : ℚ :=
  t.steps.foldl (fun acc s => acc + s.costDefect) 0

constant RVAccept : {X : Type} → Trace X → Prop

def emptyTrace {X : Type} (x : X) : Trace X :=
{ steps := [],
  chain := by
    intro i h
    cases Nat.not_lt_zero _ h }

def concat {X : Type} (t₁ t₂ : Trace X) : Trace X :=
{ steps := t₁.steps ++ t₂.steps,
  chain := by
    intro i h
    sorry }

structure CohMor (X : Type) (V : X → ℚ) (x y : X) where
  trace  : Trace X
  rv_ok  : RVAccept trace
  valid  : V y + traceSpend trace ≤ V x + traceDefect trace

axiom rv_id :
  ∀ {X : Type} (x : X), RVAccept (emptyTrace x)

axiom rv_comp :
  ∀ {X : Type} (t₁ t₂ : Trace X),
    RVAccept t₁ → RVAccept t₂ → RVAccept (concat t₁ t₂)
