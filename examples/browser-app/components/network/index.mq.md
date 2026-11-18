---
markup: occlusion
title: Network panel assembly
import:
  socket: socket.mq.md
  controls: ../controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

Network places plain explanatory content, the actual response output, a batch-fetch action and a local echo group. Batch requests retain the original public demonstration endpoint and may fail; ordered callback records show each status and body. Socket transport is local and independent. The panel starts hidden and the browser client alone selects it.

We **heading = [Describe the network responsibility](component.markdown(text="## 网络\n\n批量请求显示每项的实际状态；WebSocket 使用本机 /live 回声服务。"))** and **output = [Describe literal transport feedback](controls.text(id="net-out", value="idle"))** and **fetch = [Describe the batch request action](controls.button(id="fetch-all", label="fetch_all ×2 uuid"))** and **echo = [Assemble local socket controls](socket.nodes())** and **first = [Place feedback below its explanation](table.concat(a=`heading`, b=`output`))** and **last = [Keep fetch before socket interaction](table.concat(a=`fetch`, b=`echo`))** and **children = [Preserve the two network responsibilities](table.concat(a=`first`, b=`last`))**.

`attrs` =

| id | hidden |
|---|---|
| panel-net | True |

Return *[Describe the initially hidden Network panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="network"))*.
