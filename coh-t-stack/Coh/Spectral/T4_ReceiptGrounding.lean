import Coh.Spectral.T4_Visibility
import Coh.Contract.Micro

namespace Coh.Spectral

open Coh.Contract

/-!
# T4 Cross-Layer Grounding: Visibility â†’ MicroReceipt Defect

This module bridges the abstract `visibility_bound` theorem (T4) with the
concrete `MicroReceipt.metrics.defect` field in `Coh.Contract`.

## Modelling Choice

The abstract defect operator `Î” : G â†’ â„` in T4 is instantiated here as
the *real-valued defect metric* of a micro-receipt:
```
Î” r = r.metrics.defect
```
The `visibility_bound` theorem then says: any receipt with nonzero defect
produces a strictly observable anomaly (a bounded-away-from-zero signal).

## Scope

This is a *pointwise* visibility statement â€” each individual defect event is
visible.  The `uniform_visibility_bound` (operator-level T4 theorem) is not
instantiated here; connecting it would require modelling a family of receipts
as a linear operator, which is left for a future module.
-/

/-- The receipt defect function, cast to â„ for use with T4. -/
noncomputable def receiptDefect (r : MicroReceipt) : â„ :=
  (r.metrics.defect : â„)

/-- T4 Receipt Grounding: any micro-receipt with nonzero defect produces a
    strictly positive (visible) anomaly signal.

    This instantiates `visibility_bound` with `G := MicroReceipt` and
    `Î” := receiptDefect`. -/
theorem receipt_defect_implies_visibility
    (r : MicroReceipt)
    (h : r.metrics.defect â‰  0) :
    âˆƒ Îµ : â„, Îµ > 0 âˆ§ |receiptDefect r| â‰¥ Îµ := by
  apply visibility_bound (Î” := receiptDefect)
  intro heq
  apply h
  have : (r.metrics.defect : â„) = 0 := heq
  exact_mod_cast this

/-- T4 Receipt Grounding (contrapositive): if the defect anomaly is
    invisible (zero), then the receipt's defect metric is zero. -/
theorem zero_visibility_implies_zero_defect
    (r : MicroReceipt)
    (hVis : receiptDefect r = 0) :
    r.metrics.defect = 0 := by
  unfold receiptDefect at hVis
  exact_mod_cast hVis

end Coh.Spectral
