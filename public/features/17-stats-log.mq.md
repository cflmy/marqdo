---
markup: occlusion
title: Mid stdlib — stats and log fields
description: Thin mean/median/stdev + structured log fields (Mid2 M10)
import:
  stats: lib/stats.mq.md
  log: lib/log.mq.md
---

# Closing Mid2 sugar


Light statistics and optional `fields=` on log lines — no remote sinks, no dataframes.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`xs` =

| |
|---|
| 10 |
| 20 |
| 30 |

We first **m = [Compute m](stats.mean(list=`xs`))** for the next step.
We now **[Display this step's result](> print text=`m`)**.

`fld` =

| step | status |
|------|--------|
| done | ok |

We now **[Call log.info](> log.info text="mid2-m10" fields=`fld`)**.
