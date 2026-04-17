import Coh.Contract.Profile
import Coh.Contract.Schema
import Coh.Contract.RejectCode
import Coh.Contract.Micro
import Coh.Contract.Canon
import Coh.Contract.Slab
import Coh.Contract.SlabV2
import Coh.Contract.TestVectors
import Coh.Contract.Boundary
import Coh.Contract.MicroV2

/-!
Public contract theorem surface.

Primary referee-facing result:
- `rv_contract_correctness`

The imported modules also expose reject-side lemmas used to mirror individual
Rust verifier branches.
-/
