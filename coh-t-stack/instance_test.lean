import Mathlib.Data.NNRat.Defs
import Mathlib.Algebra.Order.Monoid.WithTop
import Mathlib.Order.WithBot
import Mathlib.Order.CompleteLattice

def ENNRat := WithTop NNRat

#synth OrderedAddCommMonoid ENNRat
#synth CompleteLattice ENNRat
#synth LinearOrder ENNRat
