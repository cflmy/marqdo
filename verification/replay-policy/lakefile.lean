import Lake
open Lake DSL
package replay_policy
@[default_target]
lean_lib ReplayPolicy
lean_exe replayPolicy where
  root := `Main
