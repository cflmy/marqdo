---
title: ext/web/page
description: Document → View. Sole compose path is render(nodes=) with unified View schema (ADR 0007).
import cap:ext/web/_capability.mq.md
import table:lib/table.mq.md
---

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

> cap.load
`out` =

| _type | title | route | method | body | nodes | data_source | data_order | data_where |
|-------|-------|-------|--------|------|-------|-------------|------------|------------|
| page | `title` | `route` | `method` | `body` | `nodes` | `data_source` | `data_order` | `data_where` |

*out*

## render
    + `nodes`=None
    + `db`=None

Render Document to HTML. `nodes` is the unified View table; when omitted, uses `self.nodes` or Markdown `body`.

> cap.load
**n = nodes**
1. `n` == None
  **n = [nodes](self)**
2. *
  **_ = 1**
1. `n`
  *> web_render_nodes page=`self` nodes=`n` db=`db`*
2. *
  **body = [body](self)**
  *> web_render_document page=`self` body=`body` db=`db`*

## css
    + `css`

Escape hatch — append raw CSS (not a second primary language).

> cap.load
*> web_page_css page=`self` css=`css`*
