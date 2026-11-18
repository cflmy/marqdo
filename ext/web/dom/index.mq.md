---
markup: occlusion
title: ext/web/dom
description: >-
  DOM Artifact — Markdown/body to structure; raw HTML is escape hatch only
  (ADR 0007). Never use raw as the default homepage method.
import:
  cap: ext/web/_capability/index.mq.md
  table: lib/table.mq.md
---

DOM helpers separate Markdown content from HTML rendering. Prefer
[component.markdown](../component/index.mq.md) for layout leaves. from_markdown
and raw only construct data and need no plugin; render needs --effects plugin.

```sh
marqdo call ext/web/dom/index.mq.md from_markdown 'text="# Readable"' --session handbook --toget body --json
marqdo call ext/web/dom/index.mq.md render 'dom=$body' --effects plugin --session handbook --json
```

raw remains an explicit compatibility escape hatch, not a homepage layout
method. Add content behavior through Markdown/render and preserve the existing
raw path; do not use raw to compensate for missing decomposition.

## from_markdown

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Markdown source |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | DOM map |

Use `text` as the inputs for this operation.

Turn Markdown `text` into a simple DOM map `{kind:"markdown", text}` for
`page.render` / `web.render`. This is the default body path for Document prose.

We retain **`out` = [Set the kind field in the new record](table.put(in=None, at="kind", value="markdown"))**.
Return *[Set the text field in the new record](table.put(in=`out`, at="text", value=`text`))*.

## raw

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| html | text | required | raw HTML (escape hatch) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | DOM map kind=raw |

Use `html` as the inputs for this operation.

Escape hatch — wrap raw `html`. Prefer Markdown / View nodes. Allowed only when
table-driven layout cannot express a host constraint; document the why next to
the call. Forbidden as the primary implementation of nav/sidebar/footer/home.

We retain **`out` = [Set the kind field in the new record](table.put(in=None, at="kind", value="raw"))**.
Return *[Set the html field in the new record](table.put(in=`out`, at="html", value=`html`))*.

## render

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dom | any | required | DOM map from from_markdown or raw |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML text |

Use `dom` as the inputs for this operation.

Render DOM Artifact `dom` (from from_markdown or raw) to HTML via the native
plugin. When kind is markdown,
the plugin turns Markdown text into HTML for Document preview. When kind is raw,
the stored HTML string is returned unchanged — that path exists only for the
escape hatch documented on raw, not for assembling a full site shell. Prefer
feeding markdown DOM produced by from_markdown, or skip this helper and use
page.render / web.render on View tables from file-partitioned components.

We **[Load the native plugin required by this operation](cap.load())**.
We retain **`kind` = [Read kind field](access(`dom`, "kind"))**.
1. `kind` == "raw"
  We retain **`html` = [Read html field](access(`dom`, "html"))**.
  Return *html*.
2. *
  We retain **`text` = [Read text field](access(`dom`, "text"))**.
  Return *[Render DOM Artifact dom (from from_markdown or raw) to HTML via the native plugin](web_dom_markdown(text=`text`))*.
