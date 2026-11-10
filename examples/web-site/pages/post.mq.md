---
markup: occlusion
type: web
view: main
route: /post/{slug}
title: Article · Marqdo Web
import:
  layout: ../components/index.mq.md
  intro: ../components/intro.mq.md
  articles: ../components/articles.mq.md
  component: ext/web/component/index.mq.md
  form: ext/data/form.mq.md
  table: lib/table.mq.md
---

This Document explicitly declares /post/{slug}. Its introduction ends in a Markdown leaf and its frame is shared. Request resources retain their own data or submission responsibilities. No startup page function initializes the database or starts another listener.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Page-specific content and request resources within the shared frame |

Assembly reads Markdown with fs.read and creates pure descriptors. The actual path capture selects the current database record at request time; full Markdown body is separate from list summary, and query strings cannot replace the route slug.

Page structure stays stable while request-dependent resources observe current data and identity. A missing required resource remains a diagnostic rather than a fake successful response.

We **body = [Read this page's terminal introduction](intro.nodes(name="post"))**.
We **records = [Describe the current full-body slug match](articles.detail())** and **body = [Retain the actual record resource](table.concat(a=`body`, b=`records`))**.
Return *[Place the page in its selected recursive frame](layout.frame(body=`body`, management=False))*.
