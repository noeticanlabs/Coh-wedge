import Lake
open Lake DSL

package «coh-t-stack» where
  -- add package configuration options here

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.7.0"

@[default_target]
lean_lib «Coh» where
  -- add library configuration options here
