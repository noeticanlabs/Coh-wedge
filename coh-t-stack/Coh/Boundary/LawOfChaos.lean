import Mathlib.Algebra.Order.Monoid.Defs

namespace Coh.Boundary

/--
The Law of Chaos formalizes forward admissible generation.
A transition (g, p, g') is admissible if it satisfies the hard compatibility relation
and the resource inequality: M(g') + C(p) ≤ M(g) + D(p).
-/
structure ChaosObject (G P R : Type) [OrderedAddCommMonoid R] where
  Gamma : G → P → G → Prop
  M : G → R
  C : P → R
  D : P → R

def ChaosAdmissible {G P R : Type} [OrderedAddCommMonoid R] 
  (obj : ChaosObject G P R) (g : G) (p : P) (g' : G) : Prop :=
  obj.Gamma g p g' ∧ obj.M g' + obj.C p ≤ obj.M g + obj.D p

end Coh.Boundary
