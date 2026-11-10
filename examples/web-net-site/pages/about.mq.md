---
markup: occlusion
type: web
view: main
route: /about
title: About
import:
  component: ext/web/component/index.mq.md
  layout: ../components/index.mq.md
  fs: lib/fs.mq.md
---

This Document declares its actual /about route. Its terminal explanation stays in Markdown and its parent frame owns navigation, side rail and footer. No data query or server setup occurs while reading this explanatory page.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Shared site frame containing this page's explanation |

The server assembles these nodes once, then resolves request resources only while serving. Reading requires fs.read. Follow the shared layout imports to change one region without reproducing an HTML shell here. The route remains discoverable from metadata rather than an implicit mutation chain.

We **text = [Read the page's terminal explanation](fs.read_text(path="components/content/about.md"))** and **body = [Describe ordinary Markdown content](component.markdown(text=`text`, slot="intro"))**.
Return *[Place this page within the shared recursive frame](layout.frame(body=`body`))*.
