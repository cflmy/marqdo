---
markup: occlusion
title: SPA homepage assembly
import:
  nav: nav.mq.md
  panels: panels/index.mq.md
  controls: controls.mq.md
  client: ext/web/client/index.mq.md
  table: lib/table.mq.md
---

The homepage composes navigation, heading, independent panels, status and the official runtime Resource.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Complete structured homepage without raw HTML |

This root contains component relationships only. Navigation and panels are ordinary child modules; their content and controls are already simple leaves. The official client runtime is a script Resource referencing built static assets, so no page HTML or author JavaScript is needed. The title and log use stable text targets. A valid node tree does not prove the WASM asset exists; build it and mount the static directory before opening the page.

We **navigation = [Assemble the navigation component](nav.nodes())** and **title = [Describe the changing page heading](controls.text(id="title", value="Items", kind="title"))** and **content = [Assemble the independent panels](panels.nodes())** and **log = [Describe the status text leaf](controls.text(id="log", value="spa ready", kind="pre"))** and **runtime = [Reference the official client program](client.nodes(source="/static/client.mq.md"))**.
We **head = [Keep navigation before the heading](table.concat(a=`navigation`, b=`title`))** and **body = [Keep panels after the heading](table.concat(a=`head`, b=`content`))** and **visible = [Place feedback after the panels](table.concat(a=`body`, b=`log`))**.
Return *[Retain the runtime as a separate Resource](table.concat(a=`visible`, b=`runtime`))*.
