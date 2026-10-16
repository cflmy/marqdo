---
title: ext/ai/sandbox
description: >-
  CoW trial workspace for AI-generated Marqdo: open / run / diff / commit.
  Native work lives in plugins/agent (`sandbox_*` → host_query).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

Load the agent plugin once per call (same pattern as other `ext/ai` helpers).

## ensure_plugin

**p = plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/ai/sandbox: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)
  > sys.exit code=1

## open
    + `root`=None

Open a CoW session. Optional `root=` (directory). Default: entry file directory.

> ensure_plugin

1. `root`
  *> sandbox_open root=`root`*
2. *
  *> sandbox_open*

## write
    + `id`
    + `path`
    + `text`

Write text into the session upper layer (host tree unchanged until `commit`).

> ensure_plugin
*> sandbox_write id=`id` path=`path` text=`text`*

## read
    + `id`
    + `path`

Read through CoW (upper, else lower; whiteout misses).

> ensure_plugin
*> sandbox_read id=`id` path=`path`*

## remove
    + `id`
    + `path`

Logical delete (whiteout).

> ensure_plugin
*> sandbox_remove id=`id` path=`path`*

## diff
    + `id`

List of `{ path, kind }` pending changes (`write` | `remove`).

> ensure_plugin
*> sandbox_diff id=`id`*

## commit
    + `id`

Apply upper / whiteout to the real workspace root.

> ensure_plugin
*> sandbox_commit id=`id`*

## discard
    + `id`

Drop the session without writing the host tree.

> ensure_plugin
*> sandbox_discard id=`id`*

## run
    + `id`
    + `source`=None
    + `path`=None

Run Marqdo under sandbox caps (`exec` / `net` / `plugin` off; fs via CoW).
Pass `source=` or `path=` (relative to session root).

> ensure_plugin

1. `source`
  *> sandbox_run id=`id` source=`source`*
2. `path`
  *> sandbox_run id=`id` path=`path`*
3. *
  > print text=ext/ai/sandbox.run: need source= or path=
  > sys.exit code=1

## inspect
    + `id`

Caps defaults, change counts, and availability notes.

> ensure_plugin
*> sandbox_inspect id=`id`*
