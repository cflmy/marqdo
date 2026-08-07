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

*`n` = > table.rows xs=`xs`*

> print text=`n`

*`r` = > table.row_at xs=`xs` i=0*

> print text=`r`
