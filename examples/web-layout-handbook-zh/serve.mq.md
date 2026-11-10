---
markup: occlusion
import:
  网页: ext/web/网页.mq.md
---

# main

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| 端口 | any | 18081 | 本地 HTTP 监听端口；服务启动后持续运行直到调用方停止 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

端口 `端口`=18081 可通过 --bind 覆盖。启动器不包含页面布局。

We first **监听端口 = [Compute 监听端口](整数(值=`端口`))** for the next step.
We now **[Call 网页.服务](> 网页.服务 根="." 主机="127.0.0.1" 端口=`监听端口`)**.
