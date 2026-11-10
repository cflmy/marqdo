---
markup: occlusion
type: web
view: main
route: /new
title: Publish
import:
  component: ext/web/component/index.mq.md
  layout: ../components/index.mq.md
  fs: lib/fs.mq.md
  form: ext/data/form.mq.md
  table: lib/table.mq.md
---

This Document declares its actual /new route. Its terminal explanation stays in Markdown and its parent frame owns navigation, side rail and footer. The form is referenced by its explicit mounted id; the resource and real submission policy are defined outside this page.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Shared site frame containing this page's explanation and mounted form reference |

The server assembles these nodes once, then resolves request resources only while serving. Reading requires fs.read. The current session's CSRF token and return path are supplied by the form renderer, so the page cannot embed a stale startup token. Missing form registration is a real request error. The route remains discoverable from metadata rather than an implicit mutation chain.

We **text = [Read the page's terminal explanation](fs.read_text(path="components/content/new.md"))** and **body = [Describe ordinary Markdown content](component.markdown(text=`text`, slot="intro"))**.
We **selected = [Reference the actual mounted article form](form.reference(id="article"))** and **controls = [Place that request resource](component.resource(value=`selected`, slot="article-form"))** and **body = [Keep instructions before the form](table.concat(a=`body`, b=`controls`))**.
Return *[Place this page within the shared recursive frame](layout.frame(body=`body`))*.
