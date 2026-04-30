import Mathlib.Data.NNRat.Defs
import Mathlib.Algebra.Order.Monoid.WithTop
import Mathlib.Order.WithBot
import Mathlib.Order.CompleteLattice
import Mathlib.Algebra.Order.Monoid.Canonical.Defs

abbrev ENNRat := WithTop NNRat

#synth CanonicallyOrderedAddCommMonoid NNRat
#synth OrderedAddCommMonoid ENNRat
#synth CompleteLattice ENNRat
#synth LinearOrder ENNRat
