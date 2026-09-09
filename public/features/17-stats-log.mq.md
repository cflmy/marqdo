---
title: Mid stdlib — stats and log fields
description: Thin mean/median/stdev + structured log fields (Mid2 M10)
import stats:lib/stats.mq.md
import log:lib/log.mq.md
---

# Closing Mid2 sugar

Light statistics and optional `fields=` on log lines — no remote sinks, no dataframes.

# main

`xs` =

| |
|---|
| 10 |
| 20 |
| 30 |

*m = > stats.mean list=`xs`*
> print text=`m`

`fld` =

| step |
|------|
| done |

> log.info text="mid2-m10" fields=`fld`
