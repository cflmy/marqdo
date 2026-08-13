# Native plugin ABI (v2)

| | |
|---|---|
| Status | Accepted |
| Date | 2026-08-07 |
| Header | [`include/marqdo_abi.h`](../../include/marqdo_abi.h) |
| Related | [ext-agent.md](ext-agent.md) · [stdlib-modules.md](stdlib-modules.md) |

## Goals

- Optional **native** extensions as shared libraries (`.dll` / `.so` / `.dylib`).
- **Not** linked into the `marqdo` binary; load at runtime.
- Stable **C** ABI only (no cross-compiler Rust ABI for plugins).
- Official / local paths only — **no** third-party package registry.
- **v2**: allowlisted `host_query` so plugins can read entry source / call site / skill without baking agent logic into the core.

## Contract

`MARQDO_ABI_VERSION = 2` (host accepts plugins with version `1..=2`).

Every plugin exports:

| Symbol | Role |
|--------|------|
| `marqdo_plugin_abi_version` | `1` or `2` |
| `marqdo_plugin_init` | Register functions via `MarqdoHostApi` |
| `marqdo_plugin_shutdown` | Teardown |

Host provides `register_fn`, `alloc`, `free`, and (v2) `host_query`.

### Wire format

Arguments and results are **UTF-8 JSON**:

- Args: JSON object keyed by parameter names (from `params` CSV at registration).
- Result: any JSON value mappable to Marqdo `Value` (same rules as `lib/json`).

Plugin fn returns `0` + `*out_json` on success; non-zero + optional `*err_msg` on failure. Host frees both strings with the host `free`.

### `host_query` allowlist (v2)

| name | Result JSON | Notes |
|------|-------------|--------|
| `module_source` | string | Entry `.mq.md` text |
| `call_site` | `{path,function,line}` | Active call site |
| `marqdo_skill` | string | Skill pack text |
| `cwd` | string | Host `cwd` (entry file directory) for path resolution |
| `record_plot` | `{ok:true}` | Args `{svg}` + optional `path`; push SVG into host plots (view embed / CLI auto-write), same channel as math plots |

Unknown names fail. Queries are only valid during a plugin function call (host sets thread-local context).

## Marqdo surface (`lib/plugin` / `lib/插件`)

| English | Chinese | Host |
|---------|---------|------|
| `## load` (`path`) | `## 加载` (`路径`) | `host_plugin_load` |
| `## unload` | `## 卸载` | `host_plugin_unload` |
| `## list` | `## 列出` | `host_plugin_list` |
| `## native_path` (`name`) | `## 原生路径` (`名`) | `host_ext_native_path` |

```markdown
---
import plugin:lib/plugin.mq.md
---

# main

> load path=demo.dll

*`sum` = > demo_add a=1 b=2 *
> print text=`sum`
```

After `load`, registered names are callable like other functions (plugin registry lookup after fixed host fns, before user `#`/`##` defs).

**Path sandbox:** `path` resolves under the program’s cwd / `fs_root` (same as `lib/fs`). Absolute paths outside the root are rejected unless trusted (`MARQDO_EXT` / `MARQDO_AGENT_PLUGIN` / built `libagent.*`). Prefer `native_path name=agent` for the official agent plugin.

## Demo / agent

```bash
cargo build -p marqdo_plugin_demo
cargo build -p marqdo_plugin_agent
```

- Demo (ABI v1): `demo_add`, `demo_echo` — [`tests/lib/plugin-demo.mq.md`](../../tests/lib/plugin-demo.mq.md).
- Agent (ABI v2): … **OKF agent-kb** …  
- Web (ABI v2): HTTP listen + SQLite + page render (`web_listen`, `web_render`, `web_db_*`) — [`plugins/web`](../../plugins/web); used by [`ext/web/web.mq.md`](../../ext/web/web.mq.md). Lookup: `plugin.native_path name=web` / `MARQDO_WEB_PLUGIN` / `marqdo ext add web`.

## Layering

| Layer | May call `host_*` |
|-------|-------------------|
| `lib/*` | Yes (only L1 wrappers of generic L0.5) |
| `ext/*` | **No** — compose `lib/*` or ABI plugin names via `lib/plugin` |
| Application `.mq.md` | Prefer same as ext; do not add agent/OKF originals to `src/host` |

**Hard rule:** agent / OKF domain primitives live in `plugins/agent`, never in `HostFn` / `src/host/kb.rs`.

## Non-goals

- Embedding plugins in the release exe by default
- Callbacks from plugin into arbitrary Marqdo evaluation
- Arbitrary HostFn passthrough via `host_query`
- **Third-party** plugin / package registry
