# Executable replay policy

Strict protocol replay uses this Lean policy to decide request permissions,
reply acceptance and final-history exhaustion. The Rust transport adapter does
not contain a second implementation of these decisions. Ordinary recording
does not start a policy process.

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
