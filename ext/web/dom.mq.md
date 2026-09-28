---
title: ext/web/dom
description: DOM Artifact — Markdown/body to structure; raw HTML is escape hatch only (ADR 0007).
import cap:ext/web/_capability.mq.md
import table:lib/table.mq.md
---

## from_markdown
    + `text`

Turn Markdown text into a simple DOM map `{kind:"markdown", text}` for `page.render` / `web.render`.

> cap.load
**`out` = > table.put in=None at="kind" value="markdown"**
*> table.put in=`out` at="text" value=text*

## raw
    + `html`

Escape hatch — wrap a raw HTML string. Prefer Markdown / View nodes.

> cap.load
**`out` = > table.put in=None at="kind" value="raw"**
*> table.put in=`out` at="html" value=html*

## render
    + `dom`

Render a DOM Artifact (markdown or raw) to HTML via native plugin.

> cap.load
**kind = [kind](dom)**
1. `kind` == "raw"
  **html = [html](dom)**
  *html*
2. *
  **text = [text](dom)**
  *> web_dom_markdown text=`text`*
