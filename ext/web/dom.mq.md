---
markup: occlusion
title: ext/web/dom
description: >-
  DOM Artifact — Markdown/body to structure; raw HTML is escape hatch only
  (ADR 0007). Never use raw as the default homepage method.
import:
  cap: ext/web/_capability.mq.md
  table: lib/table.mq.md
---

## For AI agents

Prefer `from_markdown` and View tables from `ext/web/layout` / `component`.
`raw` is an escape hatch: if you call it for a whole page shell, you are
violating the layout law in `ext/web/index.mq.md`. Always `--effects plugin`
for render paths.

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md rules --json

*None*

## from_markdown
    + `text`

Turn Markdown text into a simple DOM map `{kind:"markdown", text}` for `page.render` / `web.render`.
This is the default body path for Document prose.

| 参数 | 类型 | 说明 |
|------|------|------|
| text | text | Markdown source |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | DOM map |

> cap.load
**`out` = > table.put in=None at="kind" value="markdown"**
*> table.put in=`out` at="text" value=text*

## raw
    + `html`

Escape hatch — wrap a raw HTML string. Prefer Markdown / View nodes. Allowed only
when table-driven layout cannot express a host constraint; document the why next
to the call. Forbidden as the primary implementation of nav/sidebar/footer/home.

| 参数 | 类型 | 说明 |
|------|------|------|
| html | text | raw HTML (escape hatch) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | DOM map kind=raw |

> cap.load
**`out` = > table.put in=None at="kind" value="raw"**
*> table.put in=`out` at="html" value=html*

## render
    + `dom`

Render DOM Artifact `dom` to HTML via the native plugin. When kind is markdown,
the plugin turns Markdown text into HTML for Document preview. When kind is raw,
the stored HTML string is returned unchanged — that path exists only for the
escape hatch documented on raw, not for assembling a full site shell. Prefer
feeding markdown DOM produced by from_markdown, or skip this helper and use
page.render / web.render on View tables from file-partitioned components.

| 参数 | 类型 | 说明 |
|------|------|------|
| dom | any | DOM map from from_markdown or raw |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML text |

> cap.load
**kind = [kind](access(dom, "kind"))**
1. `kind` == "raw"
  **html = [html](access(dom, "html"))**
  *html*
2. *
  **text = [text](access(dom, "text"))**
  *> web_dom_markdown text=`text`*
