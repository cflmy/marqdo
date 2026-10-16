---
title: ext/ai/sandbox
description: >-
  CoW trial workspace for AI-generated Marqdo: open / run / diff / commit.
  Native work lives in plugins/agent (`sandbox_*` → host_query).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
import fs:lib/fs.mq.md
---

Load the agent plugin once per call (same pattern as other `ext/ai` helpers).

Capability profiles are GFM tables under `## profile` (readable authority; `open` / `inspect` echo them). Do not invent a second table dialect.

## ensure_plugin

**p = plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/ai/sandbox: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)
  > sys.exit code=1

## profile
    + `name`=default

Named caps row as a map table (`name`, `fs_write`, `exec`, `net`, `plugin`). Unknown names → `default`.

Catalog (documentation + authority):

| name | fs_write | exec | net | plugin |
|------|----------|------|-----|--------|
| default | true | false | false | false |
| readonly | false | false | false | false |

1. `name` == "readonly"
  **row = **

  | name | fs_write | exec | net | plugin |
  |------|----------|------|-----|--------|
  | readonly | false | false | false | false |

2. *
  **row = **

  | name | fs_write | exec | net | plugin |
  |------|----------|------|-----|--------|
  | default | true | false | false | false |

*row*

## open_native
    + `root`=None

Native CoW open (plugin ABI italic return). Prefer `## open` for the profile-aware facade.

> ensure_plugin

1. `root`
  *> sandbox_open root=`root`*
2. *
  *> sandbox_open*

## open
    + `root`=None
    + `profile`=default

Open a CoW session. Optional `root=` (directory). Default: entry file directory.
`profile=` selects the caps row from `## profile` (echoed by `inspect`).

**sess = open_native root=`root`**
**prof = profile name=`profile`**
**sess = table.put in=`sess` at="profile" value=`prof`**
*sess*

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

## inspect_native
    + `id`

> ensure_plugin
*> sandbox_inspect id=`id`*

## inspect
    + `id`
    + `profile`=default

Native caps / change counts, plus the GFM `## profile` row under key `profile`.

**raw = inspect_native id=`id`**
**prof = profile name=`profile`**
**out = table.put in=`raw` at="profile" value=`prof`**
*out*

## report
    + `id`
    + `path`=sandbox-diff.md

O4: write a markdown review of `## diff` for `marqdo view` / HITL (static report file, not a UI rewrite).

**pending = diff id=`id`**
**n = len value=`pending`**
**body = "# Sandbox diff report\n\nPending changes: "**
**body = body + n**
**body = body + "\n\n| path | kind |\n|------|------|\n"**
- [row](pending)
  **p = [path](row)**
  **k = [kind](row)**
  **body = body + "| "**
  **body = body + p**
  **body = body + " | "**
  **body = body + k**
  **body = body + " |\n"**
> fs.write_text path=`path` text=`body`

**out = **

| path | count |
|------|-------|
| `path` | `n` |

*out*
