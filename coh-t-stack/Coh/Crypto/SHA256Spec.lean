import Coh.Crypto.Bytes
import Coh.Core.Hash

namespace Coh.Crypto

open Coh.Core

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
