---
title: lib/text — strings
description: Trim, split, join (code-as-docs pilot)
import text:lib/text.mq.md
---

# main

Import lib/text. Bracket path or classic dotted calls both work.

**`t` = [text.str_trim] s="  marqdo  "**
**打印 内容=`t`**

**`parts` = > text.str_split s="a,b,c" sep=","**
**`j` = > text.str_join xs=`parts` sep="-"**
**打印 内容=`j`**
