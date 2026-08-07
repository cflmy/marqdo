---
title: subtask spawn fn
> lib/subtask.mq.md
> lib/json.mq.md
---

# main

*`args` = > parse text={"n":21} *

*`id` = > spawn fn=worker args=`args` *

*`v` = > wait id=`id` *

> print text=`v`

---

## worker
    + `n`

**`n` * 2**
