---
markup: occlusion
title: ext/web/page
description: >-
  Document → View. Sole compose path is render(nodes=) with unified View schema
  (ADR 0007). File-partition layouts; no HTML injection as default.
import:
  cap: ext/web/_capability.mq.md
  table: lib/table.mq.md
---

## For AI agents

Build Documents with Metadata (`type: web`) and thin pages. Put chrome in
`components/*.mq.md` files. View schema is only `|type|slot|value|attrs|style|`.
Call `ext/web/layout/index.mq.md` `rules` before inventing structure. Always
`--effects plugin` for render.

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md view_schema --json

*None*

# page
    + `title`=""
    + `route`="/"
    + `method`="GET"
    + `body`=""
    + `nodes`=None
    + `data_source`=""
    + `data_order`=""
    + `data_where`=""

Web Document handle. Prefer constructing from Artifact Metadata via `web.page` facade.
View tables use only `|type|slot|value|attrs|style|` (ZH `|类型|槽|值|属性|样式|`).
Do not pass a raw HTML string as the primary `body` for a full site shell.

> cap.load
`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

*out*

## render
    + `nodes`=None
    + `db`=None

Render this Document to HTML for preview or response. Optional `nodes` is the
unified View table (type/slot/value/attrs/style). When `nodes` is None, the
method uses self.nodes if present, otherwise Markdown body. Optional `db`
supplies bound rows for data_source pages. Prefer assembling `nodes` from
file-partitioned components via component.merge — do not pass a handmade HTML
string as the page shell. This is the sole compose path (ADR 0007); there is no
compose_main or fragment DSL.

| 参数 | 类型 | 说明 |
|------|------|------|
| nodes | any | View table or None |
| db | any | optional database handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or render payload |

> cap.load
**n = nodes**
1. `n` == None
  **n = [nodes](access(self, "nodes"))**
2. *
  **_ = 1**
1. `n`
  *> web_render_nodes page=`self` nodes=`n` db=`db`*
2. *
  **body = [body](access(self, "body"))**
  *> web_render_document page=`self` body=`body` db=`db`*

## css
    + `css`

Escape hatch — append raw CSS string `css` to this page. Named style tables and
the View style column remain the default theme path. Use this only when a host
constraint cannot be expressed as a named style row, and write that reason next
to the call. Forbidden as the primary way to style nav, sidebar, or footer.

| 参数 | 类型 | 说明 |
|------|------|------|
| css | text | raw CSS (escape hatch) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated page handle |

> cap.load
*> web_page_css page=`self` css=`css`*
