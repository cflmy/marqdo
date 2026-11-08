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

| 参数 | 类型 | 说明 |
|------|------|------|
| title | text | document title |
| route | text | path |
| method | text | HTTP method |
| body | text | Markdown body |
| nodes | any | View table or None |
| data_source | text | table name |
| data_order | text | order key |
| data_where | text | filters |

Construct a Document handle with `title`="", `route`="/", `method`="GET",
`body`="", optional View `nodes`=None, and flat `data_source`="", `data_order`="",
`data_where`="" keys. Prefer Artifact discovery via serve for production pages;
use this tool for previews and stepwise agent sessions. nodes must be the
unified View schema, never a private dialect or an HTML string shell.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | page handle |

> cap.load
`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

*out*

## render

| 参数 | 类型 | 说明 |
|------|------|------|
| page | any | Document handle |
| nodes | any | View table or None |
| db | any | optional database handle |

Render Document handle `page` to HTML for preview or response. Optional
`nodes`=None is the unified View table; when None, uses the page's own nodes or
Markdown body. Optional `db`=None is retained for facade compatibility. This offline renderer
does not populate data_source; automatic row binding belongs to served ordinary
Documents, or callers can materialize their content explicitly.
Prefer assembling nodes from file-partitioned components — do not pass a
handmade HTML string as the page shell. This is the sole compose path (ADR
0007); there is no compose_main or fragment DSL. Agents should call this free
tool (not a method chain) under `--effects plugin`.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or render payload |

> cap.load
**n = nodes**
1. `n` == None
  **n = [nodes](access(page, "nodes"))**
2. *
  **_ = 1**
**title = [title](access(page, "title"))**
1. `n`
  *> web_render_nodes page=`page` nodes=`n` db=`db` title=`title`*
2. *
  **body = [body](access(page, "body"))**
  *> web_render_document page=`page` body=`body` db=`db` title=`title`*

# page

| 参数 | 类型 | 说明 |
|------|------|------|
| title | any | See prose default / 见叙述默认值 |
| route | any | See prose default / 见叙述默认值 |
| method | any | See prose default / 见叙述默认值 |
| body | any | See prose default / 见叙述默认值 |
| nodes | any | See prose default / 见叙述默认值 |
| data_source | any | See prose default / 见叙述默认值 |
| data_order | any | See prose default / 见叙述默认值 |
| data_where | any | See prose default / 见叙述默认值 |

Inputs: `title`="", `route`="/", `method`="GET", `body`="", `nodes`=None, `data_source`="", `data_order`="", `data_where`=""

Web Document handle for author method chains. Prefer constructing from Artifact
Metadata via `web.page` facade or the free `new` tool above. View tables use
only `|type|slot|value|attrs|style|` (ZH `|类型|槽|值|属性|样式|`). Do not pass a
raw HTML string as the primary `body` for a full site shell.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Constructed value / 构造结果 |

> cap.load
`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

*out*

## render

| 参数 | 类型 | 说明 |
|------|------|------|
| nodes | any | View table or None |
| db | any | optional database handle |

Inputs: `nodes`=None, `db`=None

Render this Document to HTML for preview or response. Optional `nodes` is the
unified View table (type/slot/value/attrs/style). When `nodes` is None, the
method uses self.nodes if present, otherwise Markdown body. Optional `db` is retained for compatibility; automatic data_source binding
belongs to ordinary served Documents, not this offline renderer. Prefer assembling `nodes` from
file-partitioned components via component.merge — do not pass a handmade HTML
string as the page shell. This is the sole compose path (ADR 0007); there is no
compose_main or fragment DSL.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or render payload |

> cap.load
**n = nodes**
1. `n` == None
  **n = [nodes](access(self, "nodes"))**
2. *
  **_ = 1**
**title = [title](access(self, "title"))**
1. `n`
  *> web_render_nodes page=`self` nodes=`n` db=`db` title=`title`*
2. *
  **body = [body](access(self, "body"))**
  *> web_render_document page=`self` body=`body` db=`db` title=`title`*

## css

| 参数 | 类型 | 说明 |
|------|------|------|
| css | text | raw CSS (escape hatch) |

Inputs: `css`

Escape hatch — append raw CSS string `css` to this page. Named style tables and
the View style column remain the default theme path. Use this only when a host
constraint cannot be expressed as a named style row, and write that reason next
to the call. Forbidden as the primary way to style nav, sidebar, or footer.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated page handle |

> cap.load
*> web_page_css page=`self` css=`css`*
