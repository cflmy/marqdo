---
markup: occlusion
title: Collaborative page assembly
import:
  component: ext/web/component/index.mq.md
  client: ext/web/client/index.mq.md
  editor: editor.mq.md
  status: status.mq.md
  fs: lib/fs.mq.md
  table: lib/table.mq.md
---

The page composes a terminal Markdown introduction, intrinsic native controls and an official runtime Resource.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Complete structured collaborative page |

The introduction is read as ordinary Markdown from intro.md within the granted site root. The editor and status are independent control leaves, while the script Resource refers to the actual built bridge and WASM files. This assembly describes one page and starts no server or socket. Transport policy remains in serve and client behavior remains in static/client.mq.md.

We **text = [Read the terminal Markdown introduction](fs.read_text(path="intro.md"))** and **intro = [Describe the introduction as Markdown](component.markdown(text=`text`, slot="intro"))** and **field = [Assemble the independent native editor](editor.nodes())** and **feedback = [Assemble presence and connection status](status.nodes())** and **runtime = [Reference the official client program](client.nodes(source="/static/client.mq.md"))**.
We **body = [Keep introduction before the editor](table.concat(a=`intro`, b=`field`))** and **visible = [Place independent feedback after the field](table.concat(a=`body`, b=`feedback`))**.
Return *[Retain runtime loading as a separate resource](table.concat(a=`visible`, b=`runtime`))*.
