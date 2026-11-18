---
markup: occlusion
title: Public and desk layout entry
import:
  component: ext/web/component/index.mq.md
  nav: nav.mq.md
  side: side.mq.md
  foot: foot.mq.md
  desk: desk.mq.md
  head: head.mq.md
  table: lib/table.mq.md
---

This parent declares one site frame. The public tree has navigation, side/main and footer; the desk has its own navigation and footer with no public rail. Children own Markdown or typed request resources. Geometry is declared here and compiled from the original theme tables, never hidden inside a rendered HTML string.

## frame

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| body | list | required | Page-owned main nodes, retaining nested resources |
| management | bool | False | Select the gated custom desk frame without a public side rail |
| homepage | bool | False | Keep original home SEO scoped to Home |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Recursive site structure and pure head resources |

Use `body`, `management`=False and `homepage`=False. Child navigation keeps its actual session conditions; this function does not authenticate or authorize a visitor. The desk route is separately gated in serve, so choosing its frame cannot grant access. Footer content is already a simple Markdown responsibility and needs no further internal layout.

The stable slots nav, sidebar, main and footer retain fragment identities. Native resource wrappers remain in the View tree; the theme transparently places their semantic children in the original grid. The class is-desk is local to this frame, replacing the old whole-body mutation. The theme projection keeps its original rules and values under that explicit scope. Pure head resources are collected into the real document head.

We **top = [Assemble public navigation](nav.nav())** and **rail = [Assemble the public side rail](side.side())** and **bottom = [Read the public Markdown footer](foot.foot())** and **classes = "site-layout has-sidebar"**.
1. `management`
  We **top = [Use the desk's own navigation](desk.nav())** and **rail = [Keep the desk free of the public rail](table.empty_list())** and **bottom = [Read the desk return links](desk.foot())** and **classes = "site-layout no-sidebar is-desk"**.
We **main = [Retain the supplied main responsibility](component.wrap(children=`body`, kind="section", slot="main", style="main"))** and **children = [Keep navigation before the public rail](table.concat(a=`top`, b=`rail`))** and **children = [Keep main content after navigation](table.concat(a=`children`, b=`main`))** and **children = [Finish with the selected footer](table.concat(a=`children`, b=`bottom`))**.
We **frame = [Declare the shared named layout region](component.wrap(children=`children`, kind="section", slot="site", style=`classes`))** and **resources = [Retain actual SEO, fonts and compiled theme](head.nodes(homepage=`homepage`))**.
Return *[Keep head declarations alongside the recursive frame](table.concat(a=`resources`, b=`frame`))*.
