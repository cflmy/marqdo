---
markup: occlusion
title: ext/web/page
description: >-
  Document → View. Sole compose path is render(nodes=) with unified View schema
  (ADR 0007). File-partition layouts; no HTML injection as default.
import:
  cap: ext/web/_capability/index.mq.md
  table: lib/table.mq.md
---

Documents hold body text or nested View rows. For reusable layout follow
[layout](../layout/index.mq.md); render is the shared offline path. A complete
session begins with new, saves the handle, then reads it with a protected reference.

```sh
marqdo call ext/web/page/index.mq.md new 'title="Preview"' 'body="# Hello"' --effects plugin --session handbook --toget pg --json
marqdo call ext/web/page/index.mq.md render 'page=$pg' --effects plugin --session handbook --json
```

For a live structured Document declare view: main and return composed rows from
its main function; serve executes that assembly once at startup. Ordinary web
Documents retain their Markdown path. The reference site demonstrates both.
New behavior belongs here, in the Chinese mirror and the artifact renderer;
preserve tests/ext/web-artifact-render-smoke.mq.md and the handbook tests.

## new

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| title | text | "" | document title |
| route | text | "/" | path |
| method | text | "GET" | HTTP method |
| body | text | "" | Markdown body |
| nodes | any | None | View table or None |
| data_source | text | "" | table name |
| data_order | text | "" | order key |
| data_where | text | "" | filters |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | page handle |

Use `title`="", `route`="/", `method`="GET", `body`="", `nodes`=None, `data_source`="", `data_order`="", `data_where`="" as the inputs for this operation.

Construct a Document handle with `title`="", `route`="/", `method`="GET",

`body`="", optional View `nodes`=None, and flat `data_source`="", `data_order`="",

`data_where`="" keys. Prefer Artifact discovery via serve for production pages;
use this tool for previews and stepwise agent sessions. nodes must be the
unified View schema, never a private dialect or an HTML string shell.

We **[Load the native plugin required by this operation](cap.load())**.

`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

Return *out*.

## render

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| page | any | required | Document handle |
| nodes | any | None | View table or None |
| db | any | None | optional database handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or render payload |

Use `page`, `nodes`=None, `db`=None as the inputs for this operation.

Render Document handle `page` to HTML for preview or response. Optional

`nodes`=None is the unified View table; when None, uses the page's own nodes or
Markdown body. Optional `db`=None is retained for facade compatibility. This offline renderer
does not populate data_source; automatic row binding belongs to served ordinary
Documents, or callers can materialize their content explicitly.
Prefer assembling nodes from file-partitioned components — do not pass a
handmade HTML string as the page shell. This is the sole compose path (ADR
0007); there is no compose_main or fragment DSL. Agents should call this free
tool (not a method chain) under `--effects plugin`.

We **[Load the native plugin required by this operation](cap.load())**.
We retain **n = nodes**.
1. `n` == None
  We retain **`n` = [Read nodes field](access(`page`, "nodes"))**.
2. *
  **_ = 1**
We retain **`title` = [Read title field](access(`page`, "title"))**.
1. `n`
  Return *[Render Document handle page to HTML for preview or response](web_render_nodes(page=`page`, nodes=`n`, db=`db`, title=`title`))*.
2. *
  We retain **`body` = [Read body field](access(`page`, "body"))**.
  Return *[Render Document handle page to HTML for preview or response](web_render_document(page=`page`, body=`body`, db=`db`, title=`title`))*.

# page

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| title | any | "" | Document title used by preview and page rendering |
| route | any | "/" | Declared URL path at which this Document may be mounted |
| method | any | "GET" | HTTP request method for the declared route |
| body | any | "" | Markdown document content; complex sites use recursively split components |
| nodes | any | None | Nested View records, or None to retain document-body rendering |
| data_source | any | "" | Document data source declaration; empty means no automatic source |
| data_order | any | "" | Ordering declaration for the selected document data source |
| data_where | any | "" | Filter declaration for the selected document data source |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Document handle retaining title, route, method, body, nodes and data declarations |

Use `title`="", `route`="/", `method`="GET", `body`="", `nodes`=None, `data_source`="", `data_order`="", `data_where`="" as the inputs for this operation.

Web Document handle for author method chains. Prefer constructing from Artifact
Metadata via `web.page` facade or the free `new` tool above. View tables use
only `|type|slot|value|attrs|style|` (ZH `|类型|槽|值|属性|样式|`). Do not pass a
raw HTML string as the primary `body` for a full site shell.

We **[Load the native plugin required by this operation](cap.load())**.

`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

Return *out*.

## render

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| nodes | any | None | View table or None |
| db | any | None | optional database handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or render payload |

Use `nodes`=None, `db`=None as the inputs for this operation.

Render this Document to HTML for preview or response. Optional `nodes` is the
unified View table (type/slot/value/attrs/style). When `nodes` is None, the
method uses self.nodes if present, otherwise Markdown body. Optional `db` is retained for compatibility; automatic data_source binding
belongs to ordinary served Documents, not this offline renderer. Prefer assembling `nodes` from
file-partitioned components via component.merge — do not pass a handmade HTML
string as the page shell. This is the sole compose path (ADR 0007); there is no
compose_main or fragment DSL.

We **[Load the native plugin required by this operation](cap.load())**.
We retain **n = nodes**.
1. `n` == None
  We retain **`n` = [Read nodes field](access(`self`, "nodes"))**.
2. *
  **_ = 1**
We retain **`title` = [Read title field](access(`self`, "title"))**.
1. `n`
  Return *[Render this Document to HTML for preview or response](web_render_nodes(page=`self`, nodes=`n`, db=`db`, title=`title`))*.
2. *
  We retain **`body` = [Read body field](access(`self`, "body"))**.
  Return *[Render this Document to HTML for preview or response](web_render_document(page=`self`, body=`body`, db=`db`, title=`title`))*.

## css

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| css | text | required | raw CSS (escape hatch) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated page handle |

Use `css` as the inputs for this operation.

Escape hatch — append raw CSS string `css` to this page. Named style tables and
the View style column remain the default theme path. Use this only when a host
constraint cannot be expressed as a named style row, and write that reason next
to the call. Forbidden as the primary way to style nav, sidebar, or footer.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Escape hatch — append raw CSS string css to this page](web_page_css(page=`self`, css=`css`))*.
