# Executable replay policy

Strict protocol replay uses this Lean policy to decide request permissions,
reply acceptance and final-history exhaustion. The Rust transport adapter does
not contain a second implementation of these decisions. Ordinary recording
does not start a policy process.

The JSON-lines transport sends a complete `PolicyCommand` record on every
operation: `op`, `steps`, and `value` are always present. Lean structure defaults
are not missing-field defaults for the derived JSON decoder. In particular,
initialization and finish still need an empty `value`, while prepare/receive
still need an empty `steps` array:

```jsonl
{"op":"init","steps":[],"value":""}
{"op":"finish","steps":[],"value":""}
```

The Rust adapter uses one typed wire record for all four operations. Transport
failures stop the companion and include its exit status and a bounded stderr
tail, so a decoder error is not reduced to a closed-channel message. Neither
this transport nor its diagnostics replace Lean's replay verdict.

Build the companion before building or testing strict replay:

```sh
python3 verification/replay-policy/build_policy.py --lake /path/to/lake
cargo build --locked
```

The toolchain is pinned in `lean-toolchain`. The build writes `POLICY-SHA256`,
the compiled executable and `BUILD-IDENTITY.json`. Compile Rust after generating
the identity: its adapter rejects a companion with a different policy identity.
Do not modify policy sources during a build.

For deployment, install the companion next to the marqdo executable as
`marqdo-replay-policy`. `MARQDO_REPLAY_POLICY` can select a trusted installation.
Development builds also find this package's `.lake/build/bin/replayPolicy`.
An unavailable, stale or malformed companion stops strict replay; there is no
fallback to the historical handwritten Rust policy.

`ReplayPolicy/Cursor.lean` contains the executable transitions and their basic
invariants. The paper proof project imports this module and proves interaction-
program correspondence in `MarqdoProof/ReplayCursor.lean`. The compiler, process
transport, JSON/string encoding and host dispatch adapter remain explicit trust
interfaces. These files do not establish correctness of the full Rust frontend.
