import Lake
open Lake DSL

package CohTStack where
  -- add package configuration options here

lean_lib Coh where
  -- add library configuration options here

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.16.0"
