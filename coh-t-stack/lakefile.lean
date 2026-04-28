import Lake
open Lake DSL

package «coh-t-stack» where
  -- Settings applied to both it and its dependencies
  settings := #[
    '-DautoImplicit=false',
    '-DrelaxedAutoImplicit=false'
  ]

lean_lib «Coh» where
  -- add library configuration options here

@[default_target]
lean_exe «verify_formation» where
  root := `Main
