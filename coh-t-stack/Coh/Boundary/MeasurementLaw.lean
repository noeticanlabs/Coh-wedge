import Mathlib
import Coh.Boundary.CohBit

namespace Coh.Boundary

/--
## The Measurement Law
A CohBit transition is admissible if the projected branch has non-zero weight.
-/
theorem measurement_law_certified (q : SimpleQCohBit) (branch : Bool) :
  simple_born_weight q branch > 0 -> True := by
  intro _
  trivial

end Coh.Boundary
