---
title: lib/text — strings
description: Trim, split, join wrappers
import text:lib/text.mq.md
---

# main

Import lib/text.mq.md. Functions: str_trim(s), str_split(s, sep), str_join(xs, sep).

*`t` = > text.str_trim s="  marqdo  "*

> print text=`t`

*`parts` = > text.str_split s="a,b,c" sep=","*

*`j` = > text.str_join xs=`parts` sep="-"*

> print text=`j`
