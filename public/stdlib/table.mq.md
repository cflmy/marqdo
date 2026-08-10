---
title: lib/table — lists
description: Row count and index helpers
> lib/table.mq.md
---

# main

Import lib/table.mq.md. Functions: rows(xs), row_at(xs, i). Single-column tables are lists; multi-column tables are maps (see structure/07-map). Prefer `` `xs`[^1] `` for 1-based index; `row_at` stays 0-based.

`xs` =

| v |
|---|
| a |
| b |

*`n` = > table.rows xs=`xs`*

> print text=`n`

*`r` = > table.row_at xs=`xs` i=0*

> print text=`r`
