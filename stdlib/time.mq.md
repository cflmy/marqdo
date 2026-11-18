---
markup: occlusion
title: lib/time — clock
description: Unix time, format, parse, sleep
import:
  time: lib/time.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Unix time helpers from lib/time. Patterns use strftime.

We first **s = [Compute s](time.format(unix=0, pattern="%Y-%m-%d"))** for the next step.
**打印 内容=`s`**

We first **u = [Compute u](time.parse(text="1970-01-01 00:00:00", pattern="%Y-%m-%d %H:%M:%S"))** for the next step.
**打印 内容=`u`**

**[time.sleep_ms] ms=0**
