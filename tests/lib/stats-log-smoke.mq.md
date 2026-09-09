---
title: lib/stats + log fields M10 smoke
import stats:lib/stats.mq.md
import log:lib/log.mq.md
---

# main

`xs` =

| |
|---|
| 1 |
| 2 |
| 3 |
| 4 |

*m = > stats.mean list=`xs`*
> print text=`m`

*med = > stats.median list=`xs`*
> print text=`med`

*s = > stats.stdev list=`xs`*
> print text=`s`

`fld` =

| user | n |
|------|---|
| alice | 3 |

> log.info text="hi" fields=`fld`
