---
title: quiet file subtask captures stdout/stderr
import subtask:lib/subtask.mq.md
import json:lib/json.mq.md
---

# main

*id = > subtask.spawn path=subtask-child-print.mq.md*
*waited = > subtask.wait id=`id`*
*code = > json.get value=`waited` key=code*
*val = > json.get value=`waited` key=value*
*stdout = > json.get value=`waited` key=stdout*

1. `code` == 0
  1. `val` == "child_ok"
    1. `stdout`
      > print text=quiet-io-ok
    2. *
      > print text=quiet-io-no-stdout
  2. *
    > print text=quiet-io-bad-value
2. *
  > print text=quiet-io-fail
