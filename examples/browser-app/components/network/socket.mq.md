---
markup: occlusion
title: Local echo socket controls
import:
  controls: ../controls.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

The echo group contains Open, the message input, Send and Close as separate native leaves. The server binds /live, and the official bridge sends the browser program’s transport instructions. Actual lifecycle callbacks update the network output. This component performs no handshake itself and does not rely on availability of a public echo service.

We **open = [Describe the connection action](controls.button(id="ws-open", label="WS 打开"))** and **message = [Describe exact message input](controls.input(id="ws-msg", hint="ws 消息"))** and **send = [Describe the explicit send action](controls.button(id="ws-send", label="WS 发送"))** and **close = [Describe the explicit close action](controls.button(id="ws-close", label="WS 关闭", secondary=True))** and **first = [Keep open before message entry](table.concat(a=`open`, b=`message`))** and **last = [Keep send before close](table.concat(a=`send`, b=`close`))**.
Return *[Keep the local echo interaction ordered](table.concat(a=`first`, b=`last`))*.
