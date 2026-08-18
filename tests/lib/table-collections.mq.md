---
title: table collections
description: put + list/map helpers
import table:lib/table.mq.md
---

# main

*h = > table.put in=None at="Authorization" value="Bearer-x"*
> print text=`h`[^Authorization]

`xs` =

| v |
|---|
| a |
| b |

*xs = > table.put in=`xs` at=1 value="A"*
> print text=`xs`[^1]
> print text=`xs`[^2]

`req` =

| model | messages |
|-------|----------|
| m1 | None |

*msg = > table.put in=None at="role" value="user"*
*msg = > table.put in=`msg` at="content" value="hi"*
*messages = > table.append list=None item=`msg`*
*req = > table.put in=`req` at="messages" value=`messages`*
*req = > table.put in=`req` at="stream" value=True*
> print text=`req`[^messages][^1][^content]
> print text=`req`[^stream]

*xs = > table.append list=`xs` item="c"*
*n = > table.len xs=`xs`*
> print text=`n`

*m = > table.merge a=`h` b=`msg`*
*ks = > table.keys map=`m`*
> print text=`ks`

*empty = > table.empty_map*
*sz = > table.size map=`empty`*
> print text=`sz`
