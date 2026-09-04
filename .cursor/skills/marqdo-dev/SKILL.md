---
name: marqdo-dev
description: >-
  Develop Marqdo programs and libraries the readable way: GFM tables as data,
  code-as-documentation, table.put over json.set chains, lib/ext authoring.
  Use when writing or refactoring .mq.md, creating lib/* or ext/*, reviewing
  Marqdo style, fixing unreadable json glue, or when the user asks how to
  develop Marqdo / 如何开发 marqdo.
---

# Marqdo development (author + library)

Read this **before** writing or expanding `.mq.md`. Syntax basics stay in [marqdo](../marqdo/SKILL.md); this skill is the **style and library craft** contract.

## Non-negotiable: code is documentation

A `.mq.md` file must stay readable as Markdown. If a human cannot skim the data and intent without decoding nested `json.set` / `json.append` chains, the code is wrong.

| Prefer | Avoid |
|--------|--------|
| GFM tables for maps, lists, records, wire, commands | Building the same with `json.set` / `json.append` |
| Named library helpers (`web.*`, `browser.*`, `table.*`) | Copy-paste effect-map glue in every handler |
| `table.put` for **one** dynamic key | Long `json.set` pipelines |
| `json.parse` / `json.stringify` / `json.quote` only | Using `json` as a general dict builder |

`lib/json` is for **parse / stringify / quote**. Collection edits use **GFM tables** and **`lib/table`** (`doc/design/stdlib-table.md`).

## How to shape data

### 1. GFM table geometry (memorize)

| Shape | Result |
|-------|--------|
| 1 column | list |
| ≥2 columns, **one** data row | map (headers = keys) |
| ≥2 columns, **many** rows | map of lists (column-oriented) |
| First header `@` / `行` / `row` | **list of maps** (row records) |

Cells are expressions (`doc/design/table-cell-expressions.md`): bare words = text, `` `var` `` = variable, quote paths/`/` ratios.

### 2. Good patterns

**Wire / steps / commands** — `@` record table:

```markdown
`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
```

**Multi-key return / boot** — map table with variable cells:

```markdown
`boot` =

| wire | set_text |
|------|----------|
| `wire` | `set_text` |
```

**Single dynamic key** — one `table.put` (or a helper), not a json chain:

```markdown
*`set_text` = > table.put in=None at="#log" value=msg*
**> table.put in=None at="set_text" value=set_text**
```

Better still — call a named helper:

```markdown
**> browser.set_text sel="#log" text=msg**
```

### 3. Bad pattern (do not write)

```markdown
*`ret` = > json.set map=None key="set_text" value=…*
*`ret` = > json.set map=ret key="canvas" value=…*
*`cmds` = > json.append list=cmds item=c0*
```

## Writing application `.mq.md`

1. Frontmatter: title + imports (`table`, domain libs). **Do not import `json` unless parsing strings.**
2. `# main`: state binds, then **tables** for wire/config, return boot map via table or helper.
3. `##` handlers: short prose OK; data as tables; return via `browser.*` / `web.*` / one `table.put`.
4. Blank line between comment paragraphs and code.
5. Run `marqdo run path.mq.md` from repo root.

Browser client programs: prefer `import browser:lib/browser.mq.md` (no native plugin). Server sites: `ext/web` + GFM page/style tables.

## Developing libraries (`lib/*`, `ext/*`)

### Goals

- **Authors** see tables and short calls; **library** may hide `table.put` / host calls.
- One concern per helper; name the effect (`set_text`, `canvas`, …), do not expose bag surgery.
- EN + ZH pair when shipping user-facing libs (`lib/browser.mq.md` ↔ `lib/浏览器.mq.md`, `ext/web` ↔ `ext/web/网页.mq.md`).
- `ext/**` never calls `host_*` — use plugins / public lib only (`doc/design/ext-agent.md`).

### Library checklist

- [ ] Public examples use **tables + helpers**, not json glue
- [ ] Helpers documented with a one-line purpose above `##`
- [ ] Defaults on `+` params; omit `None` fields in merged maps
- [ ] No `json.set` chains in new code — use `table.put` / `table.merge` / GFM
- [ ] `marqdo run` on a tiny demo under `examples/` or inline smoke

### Where helpers live

| Need | Place |
|------|--------|
| Browser effect maps (WASM client) | `lib/browser.mq.md` |
| Site HTML / HTTP / DB | `ext/web` classes + compose from tables |
| List/map primitives | `lib/table.mq.md` |
| Parse JSON text | `lib/json.mq.md` only |

## Refactoring existing glue

When editing a file full of `json.set`:

1. Turn static structures into GFM tables.
2. Replace repeated effect shapes with `browser.*` / `web.text_patch` / `web.dom_patch`.
3. Leave at most sparse `table.put` for dynamic keys.
4. Drop `import json:…` if unused.

## Anti-patterns

| Wrong | Right |
|-------|-------|
| `json.set` × N to build boot | GFM map table or `browser`/`web` helper |
| `json.append` to build command list | `@` record table |
| Dummy column to force a 1-key map | `table.put` or helper |
| Importing `ext/web` only for `text_patch` in WASM | `lib/browser` (no native plugin) |
| Unreadable nested maps as “clever” | Flatten into tables humans can read |

## Related

- Syntax: [marqdo/SKILL.md](../marqdo/SKILL.md) · [examples.md](../marqdo/examples.md)
- Tables: `doc/design/stdlib-table.md` · `doc/design/table-cell-expressions.md`
- Browser effects: `doc/roadmap/browser-wasm-e.md` · `doc/roadmap/browser-wasm-f.md`
- Release: [marqdo-release](../marqdo-release/SKILL.md)
