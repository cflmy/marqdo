---
markup: occlusion
title: Terminal introduction component
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
---

The introduction ends in one ordinary Markdown responsibility: category, title, explanation, feature bullets and reading steps. Markdown provides that hierarchy directly; the theme selects those semantic elements. Page geometry stays in the parent frame, and no content row uses an HTML shell or the removed intro-binding dialect.

## nodes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Author-selected home, about, new, desk or post Markdown leaf |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One semantic introduction region around the actual Markdown leaf |

Use `name` under the granted site root with fs.read. The relative leaf path is selected by the authored Document, never a request parameter. Missing leaves retain the native file diagnostic rather than displaying a copied fallback. Reading the leaf does not execute its emphasis or links as Marqdo statements; its content is data for the native Markdown renderer.

Each leaf is already simple enough for Markdown: blockquote category, heading, paragraphs, bullets and ordered steps. A change that adds independent internal geometry should split again. Style selectors preserve the original typography, badges and step panel while the page still reads as a document.

We **path = "components/content/" + name + ".md"** and **text = [Read this introduction's actual content](fs.read_text(path=`path`))** and **leaf = [Retain its simple Markdown hierarchy](component.markdown(text=`text`, slot="intro-content"))**.
Return *[Keep the introduction as a semantic region](component.wrap(children=`leaf`, kind="section", slot="intro", style="main-intro"))*.
