---
title: writeback param must not shadow writeback module
description: Regression — step/plan param writeback=True used to break > writeback.record (bool is not a map). Use a non-colliding import alias.
import wb:lib/writeback.mq.md
import writeback:lib/writeback.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# box
    + `writeback`=False

## go
    + `writeback`=False

Mimic agent.step: branch on param named writeback, then persist via aliased lib import.

*out = > json.parse text={"status":"ok","note":"shadow-probe"}*
1. `writeback`
  *body = > json.stringify value=`out`*
  *st = > json.get value=`out` key="status"*
  1. `st` == "ok"
    > wb.record value=`body` key="ok"
  2. *
    > wb.record value=`body` key="error"
2. *
  *_ = 1*
****

# main

*b = > box*
> `b`.go writeback=True
<!-- marqdo-out ok
{"note":"shadow-probe","status":"ok"}
-->

*cached = > writeback.get key="ok"*
1. `cached`
  > print text=writeback-shadow-ok
2. *
  > print text=writeback-shadow-missing
  > sys.exit code=1
