import Coh.Prelude

namespace Coh.Crypto

/-- Minimal byte-sequence model used at the spec/runtime boundary. -/
abbrev ByteSeq := String

/-- UTF-8 boundary model for the frozen receipt fragment. -/
def utf8Encode (s : String) : ByteSeq :=
  s

/-- Concatenation on modeled byte sequences. -/
def bytesConcat (xs ys : ByteSeq) : ByteSeq :=
  xs ++ ys

/-- Canonical delimiter used by the Rust hash boundary. -/
def pipeDelimiter : ByteSeq :=
  "|"

theorem utf8Encode_deterministic (s : String) :
    utf8Encode s = utf8Encode s := rfl

theorem bytesConcat_assoc (xs ys zs : ByteSeq) :
    bytesConcat (bytesConcat xs ys) zs = bytesConcat xs (bytesConcat ys zs) := by
  simp [bytesConcat, String.append_assoc]

theorem bytesConcat_with_empty_left (xs : ByteSeq) :
    bytesConcat "" xs = xs := by
  simp [bytesConcat]

theorem bytesConcat_with_empty_right (xs : ByteSeq) :
    bytesConcat xs "" = xs := by
  simp [bytesConcat]

end Coh.Crypto
