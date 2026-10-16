# Roadmap: `ext/ai/sandbox` (AI CoW trial runtime)

| | |
|---|---|
| Status | **Active** |
| Date | 2026-10-16 |
| Design | [ext-ai-sandbox.md](../design/ext-ai-sandbox.md) |
| Research | [ai-agent-sandbox.md](../research/ai-agent-sandbox.md) |

## Milestones

| ID | Deliverable | Done when |
|----|-------------|-----------|
| S0 | Design + this roadmap + research cross-links | **done** |
| S1 | CoW open/write/read/remove/diff/commit/discard | **done** (`sandbox-cow`) |
| S2 | `run` + default caps + core lib inside run | **done** (`sandbox-run-lib`) |
| S3 | `inspect` + deny plugin.load | **done** (`sandbox-deny`) |
| S4 | Polish: ai-stack + CHANGELOG | **done** |

## Gold samples (target)

- `tests/ext/sandbox-cow.mq.md`
- `tests/ext/sandbox-run-lib.mq.md`
- `tests/ext/sandbox-deny.mq.md`

## Out of scope this wave

wasmtime CLI embed · Wasm ABI plugins · bwrap/microVM · full web in sandbox · auto-wire agent.step · edit `doc/next/*`
