---
markup: occlusion
type: web
view: main
route: /about
title: About · Marqdo Web
import:
  layout: ../components/index.mq.md
  intro: ../components/intro.mq.md
  articles: ../components/articles.mq.md
  component: ext/web/component/index.mq.md
  form: ext/data/form.mq.md
  table: lib/table.mq.md
---

This Document explicitly declares /about. Its introduction ends in a Markdown leaf and its frame is shared. Request resources retain their own data or submission responsibilities. No startup page function initializes the database or starts another listener.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Page-specific content and request resources within the shared frame |

Assembly reads Markdown with fs.read and creates pure descriptors. This explanation page requires no database query or form reference. Follow the layout imports to change one region without copying a page shell.

Page structure stays stable while request-dependent resources observe current data and identity. A missing required resource remains a diagnostic rather than a fake successful response.

We **body = [Read this page's terminal introduction](intro.nodes(name="about"))**.
Return *[Place the page in its selected recursive frame](layout.frame(body=`body`, management=False))*.
