import Coh.Crypto.Bytes
import Coh.Core.Hash

namespace Coh.Crypto

open Coh.Core

/-!
## Important: Symbolic Hash Model

`sha256_spec` uses a *symbolic* placeholder for SHA-256:
```
hashBytes s = âŸ¨s!"SHA256({s})"âŸ©
```
This is **not** a cryptographic implementation.  It is a deterministic,
injective model used to verify *structural* (input-layout) refinement only.

No collision resistance, preimage resistance, or bit-level SHA-256 properties
are proved here.  The real SHA-256 computation lives in the Rust kernel
(`sha2` crate, FIPS 180-4).
-/

/-- Digest type at the crypto boundary; currently reuses the core digest carrier. -/
abbrev Digest := ChainDigest

/-- Abstract SHA-256 specification for the current symbolic model. -/
def sha256_spec (input : ByteSeq) : Digest :=
  hashBytes input

theorem sha256_spec_deterministic (input : ByteSeq) :
    sha256_spec input = sha256_spec input := rfl

theorem hashBytes_refines_sha256_spec (input : ByteSeq) :
    hashBytes input = sha256_spec input := rfl

end Coh.Crypto
