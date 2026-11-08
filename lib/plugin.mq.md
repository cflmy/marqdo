---
markup: occlusion
title: lib/plugin
description: Load optional native ABI plugins (.dll / .so / .dylib)
---

Load optional native ABI plugins (.dll / .so / .dylib).

Locate and load native capabilities.

Behavior and limits: An empty list means no plugin is loaded in this process. Loading requires plugin.call. Prefer the relevant domain index for normal work; plugins are native-only.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/plugin.rs](../src/host/plugin.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## load

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load a native plugin from path |

Use `path` as the inputs for this operation.

Effects: plugin.call.

Load a native plugin from path.

Caller: [plugin.load] path=`so`

Return *[Load a native plugin from path](> host_plugin_load path=`path`)*.

## unload

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Unload the current native plugin |

Effects: plugin.call.

Unload the current native plugin.

Return *[Unload the current native plugin](host_plugin_unload())*.

## list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | List loaded native plugins |

Effects: pure.

Call this function directly:

```sh
marqdo call lib/plugin.mq.md list  --effects none --json
```

List loaded native plugins.

Return names loaded in this host process. Listing does not load a plugin, and a new process may have an empty list.

Return *[List loaded native plugins](host_plugin_list())*.

## native_path

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Installed extension identifier, for example agent |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Resolve installed or local native plugin path (e.g |

Use `name` as the inputs for this operation.

Effects: plugin.call.

Resolve installed or local native plugin path (e.g. agent).

Return *[Resolve installed or local native plugin path (e.g](> host_ext_native_path name=`name`)*.
