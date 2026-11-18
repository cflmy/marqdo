# Roadmap: Agent/LLM 优化波次 O1–O5

| | |
|---|---|
| Status | **Active** |
| Date | 2026-10-16 |
| Design | [ext-ai-optimize.md](../design/ext-ai-optimize.md) |
| Research | [ai-agent-llm-optimize-2026-10.md](../research/ai-agent-llm-optimize-2026-10.md) |
| Prior | [ext-agent-optimize.md](ext-agent-optimize.md) (A0–A4 / Wave B) · [ext-ai-sandbox.md](../design/ext-ai-sandbox.md) |

## Principles

1. Executable `.mq.md` remains authority; OKF reuses; sandbox proves; LLM explores.
2. No hidden TOOL: loop; no vector-store memory authority; no default shell Agent.
3. Do not edit `doc/next/*`.

## Waves

| Wave | Scope | Gold / deliverable | Status |
|------|-------|--------------------|--------|
| **O1** | A1 opt-in `sandbox=` · A3 profile table · D1 skill · D4 example | `agent-sandbox-optin` · `examples/agent-sandbox-okf` | **done** |
| **O2** | A2 promote proof · B1 OKF meta · B3/E1 harness Skill Lift | `agent-sandbox-promote` · harness subset | **done** |
| **O3** | C1 step route · C2 cost · B6 EFI default | `agent-step-route` · run usage/efi | **done** |
| **O4** | A4/D2 lint · A5 diff review report | `scripts/mq-style-lint.sh` · `sandbox.report` | **done** |
| **O5** | B4 quarantine · C3 offline policy rewrite | `agent-sandbox-quarantine` · `optimize-standing-policy.sh` | **done** |

## Non-goals

F1–F5 · wasmtime CLI embed · default `sandbox=True` without ADR · edit `doc/next/*`
