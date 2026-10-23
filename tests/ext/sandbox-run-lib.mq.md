---
markup: occlusion
title: sandbox.run — core lib/json inside the trial
description: Guest fixture imports lib/json; print ok and guest stdout via link index.
import:
  sandbox: ext/ai/sandbox.mq.md
---

# main

Open a session on the fixture directory (guest programs live here).

**sb = sandbox.open root="sandbox-fixtures"**
**id = [id](access(sb, "id"))**

Run the guest that imports `lib/json` and prints `1`.

**ret = sandbox.run id=`id` path="guest-json.mq.md"**
**ok = [ok](access(ret, "ok"))**
**out = [stdout](access(ret, "stdout"))**
> print text=`ok`
> print text=`out`

> sandbox.discard id=`id`
