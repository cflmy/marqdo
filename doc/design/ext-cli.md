# Official extension installer (`marqdo ext`)

| | |
|---|---|
| Status | **Accepted (v0.1)** |
| Date | 2026-08-06 |
| Related | [ext-llm.md](ext-llm.md) · [ext-agent.md](ext-agent.md) · [ext-abi.md](ext-abi.md) |

## Commands

```text
marqdo ext list              # official catalog + installed?
marqdo ext add llm           # install into local ext directory
marqdo ext add agent         # .mq.md + platform native plugin
marqdo ext remove llm
```

## Install root

1. `MARQDO_EXT` if set  
2. Else `~/.marqdo/ext` (`%USERPROFILE%\.marqdo\ext` on Windows)

`ext/…` import resolution also searches this default user dir (see [`load.rs`](../../src/load.rs)).

## Source for `add`

1. `MARQDO_EXT_SOURCE` — directory containing `llm.mq.md` / `agent.mq.md` …  
2. Else `./ext` near cwd / next to the binary (dev: Marqdo repo `ext/`)

Native plugin for `agent`: `target/debug|release/agent.dll` (or `libagent.so` / `libagent.dylib`), or `MARQDO_EXT_SOURCE/native/`.

`ext add agent` also writes `agent.plugin` (absolute path) and `native/<lib>`; `## load_native` uses `host_ext_native_path` when `MARQDO_AGENT_PLUGIN` is unset. Plugin load may open trusted paths under the install root (sandbox allowlist).

## Catalog

| Id | Ships |
|----|-------|
| `llm` | `llm.mq.md`, `大模型.mq.md` |
| `agent` | `agent.mq.md`, `智能体.mq.md` + native |

## Tests

`tests/gold.rs`: `ext_cli_add_list_remove_llm`, `ext_cli_add_agent_with_native`.

## Non-goals

- Third-party registry / arbitrary download URLs as default  
- Merging into `lib/`
