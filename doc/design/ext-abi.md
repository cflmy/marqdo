# Native plugin ABI (v1)

| | |
|---|---|
| Status | Accepted |
| Date | 2026-08-06 |
| Header | [`include/marqdo_abi.h`](../../include/marqdo_abi.h) |

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

## Marqdo surface

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

## Demo

Workspace crate `plugins/demo` (`marqdo_plugin_demo`) builds a cdylib registering `demo_add` and `demo_echo`.

## Non-goals

- Embedding plugins in the release exe
- Callbacks from plugin into arbitrary Marqdo evaluation
- OS sandbox / seccomp
- Auto-download of plugins
