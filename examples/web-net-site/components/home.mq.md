---
markup: occlusion
title: Network-site home content
import:
  component: ext/web/component/index.mq.md
  query: ext/data/view.mq.md
  socket: live.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
---

The home body has three responsibilities: a Markdown introduction, the original WebSocket widget, and a current article list. Each remains a separate component or resource. The database query is described at startup and executed per request, so publishing an article appears without restarting the site.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Introduction, native live widget and request-time article query nodes |

Read the terminal introduction with fs.read. The live component owns only native controls and the official bridge reference; network binding belongs to serve. The query uses the application's retained articles resource and stable id ordering. It neither caches startup records nor opens a second database using a guessed relative filename.

We **text = [Read the home introduction](fs.read_text(path="components/content/home.md"))** and **intro = [Describe it as Markdown](component.markdown(text=`text`, slot="intro"))** and **live = [Assemble the native WebSocket widget](socket.nodes())**.
We **current = [Describe the current article query](query.query(source="articles", order="id", link_prefix=""))** and **articles = [Place the dynamic data resource](component.resource(value=`current`, slot="articles"))**.
We **body = [Keep introduction before interaction](table.concat(a=`intro`, b=`live`))**.
Return *[Keep the actual database list after the live widget](table.concat(a=`body`, b=`articles`))*.
