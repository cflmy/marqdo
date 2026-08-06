# Official extensions (`ext/`) — LLM

| | |
|---|---|
| Status | Accepted (v0.1.x) |
| Date | 2026-08-06 |
| Related | [objects.md](objects.md) · [ext-abi.md](ext-abi.md) · [ext-agent.md](ext-agent.md) · [ext-cli.md](ext-cli.md) |

## Scope

`ext/` holds **official optional extensions** shipped with Marqdo. They are **not** stdlib (`lib/`). Users import only what they need:

```markdown
---
> ext/llm.mq.md
---
```

Resolution: `MARQDO_EXT`, `./ext`, `ext/` next to the binary (same pattern as `MARQDO_LIB`).

Other official ext: [ext-agent.md](ext-agent.md) (**agent development framework** — layout shipped; orchestration + LLM composition on roadmap). Installer CLI (planned): [ext-cli.md](ext-cli.md). Native plugins: [ext-abi.md](ext-abi.md).

**Note:** Prefer `marqdo ext add llm` for end-user install ([ext-cli.md](ext-cli.md)). Resolving `> ext/llm.mq.md` from a repo checkout or local `ext/` still works without the installer.

## Platform prerequisites

| Capability | Surface |
|------------|---------|
| HTTPS + headers | `lib/net` → `http_post` / `headers=` map |
| Dotenv | `## load_env` in ext (or `load_dotenv` / `lib/sys`) |
| JSON quote | `lib/json` → `quote` |

## `ext/llm` / `ext/大模型` (object handles)

| English | Chinese | Role |
|---------|---------|------|
| `## load_env` | `## 加载环境` | Free function: load `.env` |
| `# llm` | `# 大模型` | Object ctor → handle map |
| `## complete` / `## chat` | `## 运行` / `## 聊天` | Methods on the handle |

```markdown
---
> ext/llm.mq.md
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
- Live complete (local only): `tests/ext/llm-complete.mq.md` + gitignored `tests/ext/.env` — do not commit API keys
