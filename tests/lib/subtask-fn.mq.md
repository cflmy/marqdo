---
title: subtask spawn fn
> lib/subtask.mq.md
> lib/json.mq.md
---

# main

*`args` = > json.parse text={"n":21} *

*`id` = > subtask.spawn fn=worker args=`args` *

*`v` = > subtask.wait id=`id` *

> print text=`v`

---

## worker
    + `n`

**`n` * 2**
