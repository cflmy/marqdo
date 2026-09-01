# Official extension installer (`marqdo ext`)

| | |
|---|---|
| Status | **Accepted (v0.1)** |
| Date | 2026-08-07 |
| Related | [ext-llm.md](ext-llm.md) · [ext-agent.md](ext-agent.md) · [ext-abi.md](ext-abi.md) |

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

Imports use paths like `import llm:ext/ai/llm.mq.md` (see [`load.rs`](../../src/load.rs)).

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

Future optional ext domains get their own subdirs (e.g. `ext/web/`, `ext/quantum/`), not mixed into `ai/`.

## Catalog

| Id | Installs under `MARQDO_EXT` |
|----|-----------------------------|
| `llm` | `ai/llm.mq.md`, `ai/大模型.mq.md` |
| `agent` | `ai/agent.mq.md`, `ai/智能体.mq.md`, **`native/libagent.so`** (or `.dylib`) |
| `web` | `web/web.mq.md`, `web/网页.mq.md`, **`native/libweb.so`** — 见 [ext-web.md](ext-web.md) · [roadmap](../roadmap/ext-web.md) |
| `quantum` | `quantum/quantum.mq.md`, `quantum/量子.mq.md`, **`native/libquantum.so`** — 见 [ext-quantum.md](ext-quantum.md) · [roadmap](../roadmap/ext-quantum.md) |

`add agent` / `add web` / `add quantum` **先定位原生插件**，再复制 `.mq.md`：

1. 在 `CARGO_TARGET_DIR`、`target/{debug,release}`、源码 `ext` 旁的 `target/`、可执行文件旁等路径查找 `lib{name}.so`（或 `.dll` / `.dylib`）
2. 找不到则自动执行 `cargo build -p marqdo_plugin_{name}`，再安装到 `MARQDO_EXT/native/`
3. 写入 `{name}.plugin` 绝对路径提示，供 `plugin.native_path` 解析

这样不会出现「只装了 `.mq.md`、运行时报 native plugin not found」的半安装状态。

```bash
# 推荐：在仓库根目录
marqdo ext add web
# 等价于找到或编好 libweb.so 后拷入 ~/.marqdo/ext/native/
```

Without a prior install, runtime still resolves via `plugin.native_path name=web` (see [ext-abi.md](ext-abi.md)): `MARQDO_WEB_PLUGIN` → `CARGO_TARGET_DIR/{debug,release}/` → cwd `target/…` → beside `marqdo` binary → `MARQDO_EXT/native/`.

## Tests

`tests/gold.rs`: `ext_cli_add_list_remove_llm`, `ext_cli_add_agent`, `ext_agent_framework_smoke`, `ext_agent_run_live` ([`tests/ext/agent-run-live.mq.md`](../../tests/ext/agent-run-live.mq.md); needs `tests/ext/.env` + built `libagent`).

## Non-goals

- Third-party registry  
- Merging into `lib/`  
- Bundling domain/task helpers in official ext
