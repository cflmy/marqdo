# Official extensions (`ext/`) — LLM

| | |
|---|---|
| Status | Accepted (v0.1.x) |
| Date | 2026-08-06 |
| Related | [objects.md](objects.md) · [ext-abi.md](ext-abi.md) · [ext-agent.md](ext-agent.md) · [ext-cli.md](ext-cli.md) |

## Scope

`ext/ai/` holds **official LLM + agent** extensions. Import paths:

```markdown
---
> ext/ai/llm.mq.md
---
```

Resolution: `MARQDO_EXT/ai/…`, repo `ext/ai/…`, cwd `ext/ai/…` (see [ext-cli.md](ext-cli.md)).

Other official ext: [ext-agent.md](ext-agent.md) / [ext-agent-plan.md](ext-agent-plan.md) (**document-driven agent** — step with default writeback / plan workbook; thin `TOOL:` loop superseded). Installer: [ext-cli.md](ext-cli.md). Native plugins: [ext-abi.md](ext-abi.md).

**Note:** Prefer `marqdo ext add llm`. Import `> ext/ai/llm.mq.md` from repo or install root.

## Platform prerequisites

| Capability | Surface |
|------------|---------|
| HTTPS + headers | `lib/net` → `http_post` / `headers=` map |
| Dotenv | `## load_env` in ext (or `load_dotenv` / `lib/sys`) |
| JSON quote | `lib/json` → `quote` |

## `ext/ai/llm` / `ext/ai/大模型` (object handles)

| English | Chinese | Role |
|---------|---------|------|
| `## load_env` | `## 加载环境` | Free function: load `.env` |
| `# llm` | `# 大模型` | Object ctor → handle map |
| `## complete` / `## chat` | `## 运行` / `## 聊天` | Methods on the handle |

```markdown
---
> ext/ai/llm.mq.md
---

# main

> load_env

*`model` = > llm *
*`reply` = > `model`.complete prompt=Say hi in one word *
> print text=`reply`
```

Environment (create a **project-local** `.env`; do not commit secrets):

```env
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini
```

Fallbacks: `MARQDO_LLM_API_KEY`, `MARQDO_LLM_BASE_URL`, `MARQDO_LLM_MODEL`.

## Tests

- Import smoke: `tests/ext/llm-import.mq.md`
- Live complete (local): `tests/ext/llm-complete.mq.md` + `tests/ext/.env`
- Live agent `## step`: `tests/ext/agent-run-live.mq.md` (DeepSeek; gold: `ext_agent_run_live`)
- Agent smoke (real `llm` handle): `tests/ext/agent-smoke.mq.md` (gold: `ext_agent_framework_smoke`)
- Live complete: `tests/ext/llm-complete.mq.md` (gold: `ext_llm_complete_live`)

