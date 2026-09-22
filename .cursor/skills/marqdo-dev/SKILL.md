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
**`set_text` = table.put in=None at="#log" value=msg**
**table.put in=None at="set_text" value=set_text**
```

Better still — call a named helper:

```markdown
**browser.set_text sel="#log" text=msg**
```

### 3. Bad pattern (do not write)

```markdown
**`ret` = json.set map=None key="set_text" value=…**
**`ret` = json.set map=ret key="canvas" value=…**
**`cmds` = json.append list=cmds item=c0**
```

## Writing application `.mq.md`

1. Frontmatter: title + imports (`table`, domain libs). **Do not import `json` unless parsing strings.**
2. `# main`: state binds, then **tables** for wire/config, return boot map via table or helper.
3. `##` handlers: short prose OK; data as tables; return via `browser.*` / `web.*` / one `table.put`.
4. Blank line between comment paragraphs and code.
5. Run `marqdo run path.mq.md` from repo root.
6. **Calls in narrative:** prefer `**结果 = 某函数 名=`x`**` (no `>` on the RHS). Bracket form `礼貌 [问候] "x"` / `**出 = [put] …**` is fine when modifiers help. Use `> …` for standalone step lists. See `examples/code-as-docs/`.
7. **Do not use `##` inside `# main` for document sectioning** — those become nested functions. Use paragraphs / `---` breaks instead.
8. **Index:** prefer `[键](集合)` / `` [`名`](集合) `` over footnote `` m[^k] ``.

Browser client programs: prefer `import browser:lib/browser.mq.md` (no native plugin). Server sites: `ext/web` + GFM page/style tables.

## Developing libraries (`lib/*`, `ext/*`)

Canonical thin-template + waves: [lib-code-as-docs.md](../../doc/design/lib-code-as-docs.md) · [roadmap/lib-ext-code-as-docs.md](../../doc/roadmap/lib-ext-code-as-docs.md).

### Goals

- **Authors** see tables and short calls; **library** may hide `table.put` / host calls.
- One concern per helper; name the effect (`set_text`, `canvas`, …), do not expose bag surgery.
- EN + ZH pair when shipping user-facing libs (`lib/browser.mq.md` ↔ `lib/浏览器.mq.md`, `ext/web` ↔ `ext/web/网页.mq.md`).
- `ext/**` never calls `host_*` — use plugins / public lib only (`doc/design/ext-agent.md`).
- Collection get: **`[key](coll)`** (no new footnote indexes). Calls in examples may use `修饰 [fn] …` or classic `>` / bold.

### Library checklist

- [ ] Purpose sentence + prose params + one example + short body
- [ ] Public examples use **tables + helpers**, not json glue
- [ ] Index with `[k](c)`; foreach `- [item](coll)`
- [ ] Helpers documented with a one-line purpose above `##`
- [ ] Defaults on params; omit `None` fields in merged maps
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

## 渐进式契约 + MLSP（Phase 2/3 落地后的开发戒律）

- **契约实现**：`src/contract.rs`（类型词汇 / 三形状契约表解析 / 合并 / 四边界校验诊断）；
  `src/parse/mod.rs::extract_function_contract`（位置 + 形状双重判定，**在升参推断之前**执行；
  绑定表与普通文档表绝不吃 ⇒ 无契约文件零回归）。契约表从 body 移除——元数据永不执行。
- **静态互查**：`src/check.rs`（`marqdo check [--json]`）——契约行 vs 升参推断形参双向漂移、
  未知类型名、错位契约；Error ⇒ 非零退出。
- **AI 工具面**：`src/mlsp.rs`（`marqdo mlsp`，行分隔 JSON）——`locate` / `syntax` / `validate` /
  `repair_targets` / `schema`。语法唯一事实来源 = `parse::CORE_CONSTRUCTS`（CI 守卫与
  `doc/design/core-surface.md` 同步）；MLSP 只加检索别名，**不定义语法**。
- **准入戒律**：新特性先过 `doc/design/layers.md` 三层归属（Language 层改动须 ADR + core-surface）；
  新契约形状/类型词汇 = 文档层扩展，进 `src/contract.rs`，不进词法/语法。
- **错误出口唯一**：`src/load.rs::label_error` 已示范——装载层/任何新代码都**不许**把
  `Diagnostic` 拍平成字符串；错误链里必须能 `Diagnostic::find` 找回结构化诊断（AI/MLSP 面）。
