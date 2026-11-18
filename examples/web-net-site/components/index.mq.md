---
markup: occlusion
title: Network-site recursive layout entry
import:
  component: ext/web/component/index.mq.md
  nav: nav.mq.md
  side: side.mq.md
  foot: foot.mq.md
  table: lib/table.mq.md
---

Read this entry before changing page geometry. Navigation, side rail and footer end in Markdown leaves; the caller supplies main content nodes. Database queries and form references remain resources inside that main region, preserving fresh request behavior without rebuilding layout or concatenating HTML.

## frame

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| body | list | required | Main content nodes supplied by the page's own component |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Ordered navigation, side/main row, footer and stylesheet resource |

Use `body` to assemble one page in the same site frame. Each child module owns its content; this parent only declares order and one side-by-side relationship. The native row layout collapses on narrow screens. No child is rendered into a string before composition. The theme URL points to a file explicitly compiled by serve before discovery starts.

We **top = [Read the navigation component](nav.nav())** and **rail = [Read the side-rail component](side.side())** and **main = [Name the supplied main region](component.wrap(children=`body`, kind="column", slot="main"))**.
We **columns = [Keep side rail and main as sibling components](table.concat(a=`rail`, b=`main`))** and **center = [Declare the shared row geometry](component.wrap(children=`columns`, kind="row", slot="page-body"))** and **bottom = [Read the closing links](foot.foot())**.
We **page = [Keep navigation before content](table.concat(a=`top`, b=`center`))** and **page = [Keep the footer after content](table.concat(a=`page`, b=`bottom`))** and **style = [Reference the compiled declarative theme](component.asset(url="/static/generated/theme.css", kind="stylesheet"))**.
Return *[Retain the style resource with the readable page structure](table.concat(a=`page`, b=`style`))*.
