---
title: lib/table — lists
description: Row count and index helpers
> lib/table.mq.md
---

# main

Import lib/table.mq.md. Functions: rows(xs), row_at(xs, i). Single-column Markdown tables are lists at runtime.

`xs` =

| v |
|---|
| a |
| b |

*`n` = > rows xs=`xs`*

> print text=`n`

*`r` = > row_at xs=`xs` i=0*

> print text=`r`
