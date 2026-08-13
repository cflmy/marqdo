---
title: subtask spawn join
import subtask:lib/subtask.mq.md
import json:lib/json.mq.md
---

# main

*`id` = > subtask.spawn path=subtask-child.mq.md quiet=False *

*`waited` = > subtask.wait id=`id` *
*`code` = > json.get value=`waited` key=code *
*`val` = > json.get value=`waited` key=value *

1. `code` == 0
  > print text=`val`
2. *
  > print text=fail

> print text=done

*`id2` = > subtask.spawn path=subtask-child.mq.md *

*`waited2` = > subtask.wait id=`id2` *
*`code2` = > json.get value=`waited2` key=code *
*`val2` = > json.get value=`waited2` key=value *

1. `code2` == 0
  1. `val2` == "child_ok"
    > print text=quiet-ok
  2. *
    > print text=quiet-bad-value
2. *
  > print text=quiet-fail
