---
title: web hosting live helpers
description: Tiny lib used by hosting live-server invoke route (nested plugin call).
import web:ext/web/web.mq.md
---

## echo
+ `msg`=""
+ `payload`=None

Nested plugin call (`web.page`) while `app.listen` is the outer plugin frame —
must not clear GLOBAL_HOST so a second HTTP invoke still works.

*pg = > web.page title=`msg`*
*title = pg[^title]*

`out` =

| ok | msg | title |
|----|-----|-------|
| true | `msg` | `title` |

**out**
