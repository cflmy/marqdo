---
title: table collections
description: put + list/map helpers
import table:lib/table.mq.md
---

# main

**h = > table.put in=None at="Authorization" value="Bearer-x"**
> print text=[Authorization](`h`)

`xs` =

| v |
|---|
| a |
| b |

**xs = > table.put in=`xs` at=1 value="A"**
> print text=[1](`xs`)
> print text=[2](`xs`)

`req` =

| model | messages |
|-------|----------|
| m1 | None |

**msg = > table.put in=None at="role" value="user"**
**msg = > table.put in=`msg` at="content" value="hi"**
**messages = > table.append list=None item=`msg`**
**req = > table.put in=`req` at="messages" value=`messages`**
**req = > table.put in=`req` at="stream" value=True**
> print text=[content]([1]([messages](`req`)))
> print text=[stream](`req`)

**xs = > table.append list=`xs` item="c"**
**n = > table.len xs=`xs`**
> print text=`n`

**m = > table.merge a=`h` b=`msg`**
**ks = > table.keys map=`m`**
> print text=`ks`

**empty = > table.empty_map**
**sz = > table.size map=`empty`**
> print text=`sz`
