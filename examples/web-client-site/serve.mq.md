---
markup: occlusion
title: web-client-site serve
import:
  web: ext/web/web.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **embed = [Compute embed](web.client_embed(source="/static/client.mq.md"))** for the next step.
We now **[Display this step's result](> print text="embed-ready")**.
We now **[Call web.serve](> web.serve root="." host="127.0.0.1" port=18090 static_dir="static")**.
