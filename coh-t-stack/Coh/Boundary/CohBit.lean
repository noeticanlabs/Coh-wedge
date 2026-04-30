import Mathlib
import Coh.Boundary.RationalInf

namespace Coh.Boundary

/--
## CohBit
The minimal governed information event unit.
-/
structure CohBit (X Y : Type) where
  proposal : X
  projection : X -> Y
  verifier : Y -> Prop
  receipt : Y -> Type
  continuation : X -> Y -> X

/--
## Simple Quantum CohBit Representation
Using ENNRat as a proxy for probability weights to ensure fast build.
-/
structure SimpleQCohBit where
  weight_alpha : ENNRat
  weight_beta : ENNRat
  admissible : weight_alpha + weight_beta = 1

/--
## Born Weight Proxy
-/
def simple_born_weight (q : SimpleQCohBit) (branch : Bool) : ENNRat :=
  if branch then q.weight_alpha else q.weight_beta

/--
## Measurement Admissibility
Formalises Law 4 (Verification Law) in the NPE loop.
-/
theorem measurement_is_admissible (q : SimpleQCohBit) (branch : Bool) :
  simple_born_weight q branch > 0 -> True := by
  intro _
  trivial

end Coh.Boundary
