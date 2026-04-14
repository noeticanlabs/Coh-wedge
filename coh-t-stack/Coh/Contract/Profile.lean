import Coh.Prelude

namespace Coh.Contract

abbrev CanonProfileHash := String
abbrev PolicyHash := String

structure CanonProfile where
  schemaTag : String
  version : String
  profileHash : CanonProfileHash
  deriving Repr, DecidableEq

end Coh.Contract
