---
markup: occlusion
type: web
view: main
route: /admin
title: Admin · Marqdo Web
import:
  layout: ../components/index.mq.md
  intro: ../components/intro.mq.md
  articles: ../components/articles.mq.md
  component: ext/web/component/index.mq.md
  form: ext/data/form.mq.md
  table: lib/table.mq.md
---

This Document explicitly declares /admin. Its introduction ends in a Markdown leaf and its frame is shared. The serving entry enforces an actual admin role gate before the desk renders. No startup page function initializes the database or starts another listener.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Page-specific content and request resources within the shared frame |

Assembly reads Markdown with fs.read and creates pure descriptors. The server resolves the mounted form using actual session CSRF and return context. It does not embed a stale startup token or a separate copy of validation rules.

The desk keeps its own navigation and footer and has no public side rail. Its form intentionally shares the public article submission endpoint; gating presentation alone would not make that endpoint private, and the example does not claim otherwise.

We **body = [Read this page's terminal introduction](intro.nodes(name="desk"))**.
We **selected = [Reference the same explicitly mounted article form](form.reference(id="article"))** and **controls = [Retain current request CSRF and validation context](component.resource(value=`selected`, slot="article-form"))** and **body = [Keep instructions before the actual form](table.concat(a=`body`, b=`controls`))**.
We **records = [Describe the same current article list](articles.cards())** and **body = [Retain the actual record resource](table.concat(a=`body`, b=`records`))**.
Return *[Place the page in its selected recursive frame](layout.frame(body=`body`, management=True))*.
