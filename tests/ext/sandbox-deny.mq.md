---
title: sandbox.run — plugin.load denied; inspect lists notes
description: >
  Guest that loads a native plugin must fail with unavailable in sandbox.
  No json.get chains — link index + a small expect table.
import sandbox:ext/ai/sandbox.mq.md
---

# main

**sb = sandbox.open root="sandbox-fixtures"**
**id = [id](sb)**

**ret = sandbox.run id=`id` path="guest-plugin.mq.md"**
**ok = [ok](ret)**
**err = [error](ret)**
> print text=`ok`

**parts = split value=`err` sep="unavailable in sandbox"**
**hits = len value=`parts`**
1. `hits` > 1
  > print text=deny-ok
2. *
  > print text=`err`

**info = sandbox.inspect id=`id`**
**notes = [notes](info)**
**nn = len value=`notes`**
1. `nn` > 0
  > print text=inspect-ok
2. *
  > print text=inspect-missing

> sandbox.discard id=`id`
