---
markup: occlusion
import:
  web: ext/web/web.mq.md
---

# main

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| port | any | 18081 | Local HTTP listening port; starting the server owns its lifecycle until stopped |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

The `port`=18081 may be overridden with --bind. This launcher owns no layout.

We first **listen_port = [Compute listen_port](int(value=`port`))** for the next step.
We now **[Call web.serve](> web.serve root="." host="127.0.0.1" port=`listen_port`)**.
