import Lake
open Lake DSL

package «coh-t-stack» where
  settings := #[
    '-DautoImplicit=false',
    '-DrelaxedAutoImplicit=false'
  ]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git"

lean_lib «Coh» where
  roots := #[`Coh]
