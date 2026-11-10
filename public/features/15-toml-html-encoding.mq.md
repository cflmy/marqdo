---
markup: occlusion
title: Mid stdlib — toml, html, base32
description: Config TOML + HTML escape + base32 (Mid2 M8)
import:
  toml: lib/toml.mq.md
  html: lib/html.mq.md
  enc: lib/encoding.mq.md
---

# Config and safe text without plugins


Parse a TOML subset for tools, escape HTML before assembly, encode with base32.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **cfg = [Compute cfg](toml.parse(text="app = \"demo\"\n[server]\nport = 8080\n"))** for the next step.
We now **[Display this step's result](> print text=[app](access(`cfg`, "app")))**.
We first **srv = [server](access(`cfg`, "server"))** for the next step.
We now **[Display this step's result](> print text=[port](access(`srv`, "port")))**.

We first **safe = [Compute safe](html.escape(text="<tag>"))** for the next step.
We now **[Display this step's result](> print text=`safe`)**.

We first **b = [Compute b](enc.base32_encode(text="ok"))** for the next step.
We now **[Display this step's result](> print text=`b`)**.
