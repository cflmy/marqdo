---
title: lib/table — collections
description: put, append, and map helpers
import table:lib/table.mq.md
---

# main

Prefer `table.put` for element updates (`at=` text key, **1-based** int, or list path). Use `json` for parse/stringify only.

*`h` = > table.put in=None at="Authorization" value="Bearer-demo"*

`xs` =

| v |
|---|
| a |
| b |

*`xs` = > table.put in=`xs` at=1 value="A"*
*`n` = > table.rows xs=`xs`*

> print text=`h`[^Authorization]
> print text=`xs`[^1]
> print text=`n`
