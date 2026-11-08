---
markup: occlusion
title: ext/ai/sandbox
description: >-
  CoW trial workspace for AI-generated Marqdo: open / run / diff / commit.
  Native work lives in plugins/agent (`sandbox_*` → host_query).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
  fs: lib/fs.mq.md
---

Load the agent plugin once per call (same pattern as other `ext/ai` helpers).

Capability profiles are GFM tables under `## profile` (readable authority; `open` / `inspect` echo them). Do not invent a second table dialect.

CoW trial operations belong here; attaching a sandbox profile to another result does not isolate execution. [The AI entry](index.mq.md) explains the relation to model and agent resources. A trial produces inspectable changes; promotion is a separate existing operation with its own checks. Keep cancellation, failed execution and explicit promotion visible, and never equate a generated source file with a committed successful outcome.

## ensure_plugin

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the required native ABI plugin; missing binaries report a diagnostic message and stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/ai/sandbox: native agent plugin not found (build marqdo_plugin_agent or marqdo ext add agent)"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

## profile

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | "default" | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Named caps row as a map table (`name`, `fs_write`, `exec`, `net`, `plugin`). Unknown names → `default`. Catalog (documentation + authority): |

Use `name`="default" as the inputs for this operation.

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

Return *row*.

## open_native

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | any | None | Explicit root directory for discovery or isolated execution |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native CoW open (plugin ABI italic return). Prefer `## open` for the profile-aware facade. |

Use `root`=None as the inputs for this operation.

Native CoW open (plugin ABI italic return). Prefer `## open` for the profile-aware facade.

We **[Load the native plugin required by this operation](ensure_plugin())**.

1. `root`
  Return *[Native CoW open (plugin ABI italic return). Prefer ## open for the profile-aware facade.](sandbox_open(root=`root`))*.
2. *
  Return *[Native CoW open (plugin ABI italic return). Prefer ## open for the profile-aware facade.](sandbox_open())*.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | any | None | Explicit root directory for discovery or isolated execution |
| profile | any | "default" | Capability profile selected for the trial |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Open a CoW session. Optional `root=` (directory). Default: entry file directory. |

Use `root`=None, `profile`="default" as the inputs for this operation.

Open a CoW session. Optional `root=` (directory). Default: entry file directory.
`profile=` selects the caps row from `## profile` (echoed by `inspect`).

We retain **`sess` = [Open native](open_native(root=`root`))**.
We retain **`prof` = [Profile](profile(name=`profile`))**.
We retain **`sess` = [Set the profile field in the new record](table.put(in=`sess`, at="profile", value=`prof`))**.
Return *sess*.

## write

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| text | any | required | Text supplied to the transformation or adapter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Write text into the session upper layer (host tree unchanged until `commit`). |

Use `id`, `path`, `text` as the inputs for this operation.

Write text into the session upper layer (host tree unchanged until `commit`).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Write text into the session upper layer (host tree unchanged until commit).](sandbox_write(id=`id`, path=`path`, text=`text`))*.

## read

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read through CoW (upper, else lower; whiteout misses). |

Use `id`, `path` as the inputs for this operation.

Read through CoW (upper, else lower; whiteout misses).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Read through CoW (upper, else lower; whiteout misses).](sandbox_read(id=`id`, path=`path`))*.

## remove

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Logical delete (whiteout). |

Use `id`, `path` as the inputs for this operation.

Logical delete (whiteout).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Logical delete (whiteout).](sandbox_remove(id=`id`, path=`path`))*.

## diff

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List of `{ path, kind }` pending changes (`write` &#124; `remove`). |

Use `id` as the inputs for this operation.

List of `{ path, kind }` pending changes (`write` | `remove`).

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[List of { path, kind } pending changes (write | remove).](sandbox_diff(id=`id`))*.

## commit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply upper / whiteout to the real workspace root. |

Use `id` as the inputs for this operation.

Apply upper / whiteout to the real workspace root.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Apply upper / whiteout to the real workspace root.](sandbox_commit(id=`id`))*.

## discard

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Drop the session without writing the host tree. |

Use `id` as the inputs for this operation.

Drop the session without writing the host tree.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Drop the session without writing the host tree.](sandbox_discard(id=`id`))*.

## run

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| source | any | None | Source text inspected or prepared by this operation |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run Marqdo under sandbox caps (`exec` / `net` / `plugin` off; fs via CoW). Pass `source=` or `path=` (relative to session root). |

Use `id`, `source`=None, `path`=None as the inputs for this operation.

Run Marqdo under sandbox caps (`exec` / `net` / `plugin` off; fs via CoW).
Pass `source=` or `path=` (relative to session root).

We **[Load the native plugin required by this operation](ensure_plugin())**.

1. `source`
  Return *[Run Marqdo under sandbox caps (exec / net / plugin off; fs via CoW). Pass source= or path= (relative to session root).](sandbox_run(id=`id`, source=`source`))*.
2. `path`
  Return *[Run Marqdo under sandbox caps (exec / net / plugin off; fs via CoW). Pass source= or path= (relative to session root).](sandbox_run(id=`id`, path=`path`))*.
3. *
  We **[Print the prepared message](print(text="ext/ai/sandbox.run: need", source="or", path=""))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.

## inspect_native

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Inspect an existing native CoW session without promoting its writes. |

Use `id` as the inputs for this operation.

Inspect an existing native CoW session without promoting its writes.

We **[Load the native plugin required by this operation](ensure_plugin())**.
Return *[Inspect an existing native CoW session without promoting its writes.](sandbox_inspect(id=`id`))*.

## inspect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| profile | any | "default" | Capability profile selected for the trial |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native caps / change counts, plus the GFM `## profile` row under key `profile`. |

Use `id`, `profile`="default" as the inputs for this operation.

Native caps / change counts, plus the GFM `## profile` row under key `profile`.

We retain **`raw` = [Inspect native](inspect_native(id=`id`))**.
We retain **`prof` = [Profile](profile(name=`profile`))**.
We retain **`out` = [Set the profile field in the new record](table.put(in=`raw`, at="profile", value=`prof`))**.
Return *out*.

## report

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| path | any | "sandbox-diff.md" | Explicit filesystem path; honor the documented filesystem root and operation effects |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | O4: write a markdown review of `## diff` for `marqdo view` / HITL (static report file, not a UI rewrite). |

Use `id`, `path`="sandbox-diff.md" as the inputs for this operation.

O4: write a markdown review of `## diff` for `marqdo view` / HITL (static report file, not a UI rewrite).

We retain **`pending` = [Diff](diff(id=`id`))**.
We retain **`n` = [Len](len(value=`pending`))**.
We retain **body = "# Sandbox diff report\n\nPending changes: "**.
We retain **body = body + n**.
We retain **body = body + "\n\n| path | kind |\n|------|------|\n"**.
- [row](遍历 pending 为 row)
  We retain **`p` = [Read path field](access(`row`, "path"))**.
  We retain **`k` = [Read kind field](access(`row`, "kind"))**.
  We retain **body = body + "| "**.
  We retain **body = body + p**.
  We retain **body = body + " | "**.
  We retain **body = body + k**.
  We retain **body = body + " |\n"**.
We **[Write the prepared text to the selected file](fs.write_text(path=`path`, text=`body`))**.

**out = **

| path | count |
|------|-------|
| `path` | `n` |

Return *out*.
