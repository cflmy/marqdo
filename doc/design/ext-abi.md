# Native plugin ABI (v1)

| | |
|---|---|
| Status | Accepted |
| Date | 2026-08-06 |
| Header | [`include/marqdo_abi.h`](../../include/marqdo_abi.h) |
| Related | [ext-agent.md](ext-agent.md) · [stdlib-modules.md](stdlib-modules.md) |

## Goals

- Optional **native** extensions as shared libraries (`.dll` / `.so` / `.dylib`).
- **Not** linked into the `marqdo` binary; load at runtime.
- Stable **C** ABI only (no cross-compiler Rust ABI for plugins).
- Official / local paths only — **no** third-party package registry.

## Contract

`MARQDO_ABI_VERSION = 1`.

Every plugin exports:

| Symbol | Role |
|--------|------|
| `marqdo_plugin_abi_version` | Must equal host’s supported version |
| `marqdo_plugin_init` | Register functions via `MarqdoHostApi` |
| `marqdo_plugin_shutdown` | Teardown |

Host provides `register_fn(name, params_csv, fn)`, `alloc`, `free`.

### Wire format

Arguments and results are **UTF-8 JSON**:

- Args: JSON object keyed by parameter names (from `params` CSV at registration).
- Result: any JSON value mappable to Marqdo `Value` (same rules as `lib/json`).

Plugin fn returns `0` + `*out_json` on success; non-zero + optional `*err_msg` on failure. Host frees both strings with the host `free`.

## Marqdo surface (`lib/plugin` / `lib/插件`)

| English | Chinese | Host |
|---------|---------|------|
| `## load` (`path`) | `## 加载` (`路径`) | `host_plugin_load` |
| `## unload` | `## 卸载` | `host_plugin_unload` |
| `## list` | `## 列出` | `host_plugin_list` |

```markdown
---
> lib/plugin.mq.md
---

# main

> load path=demo.dll

*`sum` = > demo_add a=1 b=2 *
> print text=`sum`
```

After `load`, registered names are callable like other functions (plugin registry lookup after fixed host fns, before user `#`/`##` defs).

**Path sandbox:** `path` resolves under the program’s cwd / `fs_root` (same as `lib/fs`). Absolute paths outside the root are rejected. Prefer a same-directory filename, or pass a path via a variable / env (see gold tests).

## Demo

```bash
cargo build -p marqdo_plugin_demo
# Windows: target/debug/demo.dll
# Linux:   target/debug/libdemo.so
# macOS:   target/debug/libdemo.dylib
```

Registers `demo_add(a,b)`, `demo_echo(text)`. Gold: [`tests/lib/plugin-demo.mq.md`](../../tests/lib/plugin-demo.mq.md).

Official agent layout plugin: [`plugins/agent`](../../plugins/agent) — see [ext-agent.md](ext-agent.md).

## Non-goals

- Embedding plugins in the release exe by default
- Callbacks from plugin into arbitrary Marqdo evaluation
- OS sandbox / seccomp
- **Third-party** plugin / package registry

Official install of known extensions (`marqdo ext add …`) is **planned** — see [ext-cli.md](ext-cli.md). That is not a public marketplace.