import Mathlib

namespace Coh.Extraction

/--
A computable density function for a 4-component complex spinor,
represented as 8 Float values (real and imaginary parts).
-/
def compute_density (r0 i0 r1 i1 r2 i2 r3 i3 : Float) : Float :=
  r0 * r0 + i0 * i0 +
  r1 * r1 + i1 * i1 +
  r2 * r2 + i2 * i2 +
  r3 * r3 + i3 * i3

/--
Check if density is non-negative. For Floats, it always is,
but this serves as the extracted verifiable computational check.
Returns 1 for true, 0 for false.
-/
@[export coh_check_positive_density]
def coh_check_positive_density (r0 i0 r1 i1 r2 i2 r3 i3 : Float) : UInt8 :=
  if compute_density r0 i0 r1 i1 r2 i2 r3 i3 >= 0.0 then 1 else 0

end Coh.Extraction
