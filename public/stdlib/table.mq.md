---
title: lib/table — collections
description: put, append, and map helpers (code-as-docs pilot)
import table:lib/table.mq.md
---

# main

Prefer table.put for element updates (at= text key, 1-based int, or list path).
Read with link-index get. Use json for parse/stringify only.

**`h` = [table.put] in=None at="Authorization" value="Bearer-demo"**

`xs` =

| v |
|---|
| a |
| b |

**`xs` = [table.put] in=`xs` at=1 value="A"**
**`n` = > table.rows xs=`xs`**

**打印 内容=[Authorization](h)**
**打印 内容=[1](xs)**
**打印 内容=`n`**
