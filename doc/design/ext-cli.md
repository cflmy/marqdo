# Official extension installer (`marqdo ext`)

| | |
|---|---|
| Status | **Accepted (v0.1)** |
| Date | 2026-08-06 |
| Related | [ext-llm.md](ext-llm.md) · [ext-agent.md](ext-agent.md) |

## Commands

```text
marqdo ext list
marqdo ext add llm
marqdo ext add agent
marqdo ext remove llm
marqdo ext remove agent
```

## Install root

1. `MARQDO_EXT` if set  
2. Else `~/.marqdo/ext`

Imports use paths like `> ext/ai/llm.mq.md` (see [`load.rs`](../../src/load.rs)).

## Source for `add`

1. `MARQDO_EXT_SOURCE` — directory containing `ai/llm.mq.md` …  
2. Else repo `./ext` near cwd / binary

## Layout (`ext/`)

Official extensions live under **`ext/ai/`** (LLM + agent framework only). Do not add flat `.mq.md` files at `ext/` root — keeps the tree from accumulating unrelated modules.

```text
ext/
  ai/
    llm.mq.md
    大模型.mq.md
    agent.mq.md
    智能体.mq.md
```

Future optional ext domains get their own subdirs (e.g. `ext/foo/`), not mixed into `ai/`.

## Catalog

| Id | Installs under `MARQDO_EXT` |
|----|-----------------------------|
| `llm` | `ai/llm.mq.md`, `ai/大模型.mq.md` |
| `agent` | `ai/agent.mq.md`, `ai/智能体.mq.md` |

## Tests

`tests/gold.rs`: `ext_cli_add_list_remove_llm`, `ext_cli_add_agent`, `ext_agent_framework_smoke`, `ext_agent_run_live` (needs `tests/ext/.env`).

## Non-goals

- Third-party registry  
- Merging into `lib/`  
- Bundling domain/task helpers in official ext
