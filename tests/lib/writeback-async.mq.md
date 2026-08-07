---
title: async writeback via subtask
description: Fire-and-forget persist_slot with explicit line=; join on exit.
> lib/writeback.mq.md
> lib/subtask.mq.md
> lib/json.mq.md
---

## persist_slot
    + `key`
    + `value`
    + `line`

> writeback.ensure key=ok placeholder=pending line=`line`
> writeback.ensure key=error placeholder=pending line=`line`
> writeback.record value=`value` key=`key` line=`line`
****

# main

*`anchor` = > host_outer_call_line *
<!-- marqdo-out ok
async-ok
-->




<!-- marqdo-out error
pending
-->

*`wb` = > json.parse text={"key":"ok","value":"async-ok"} *
*`wb` = > json.set map=`wb` key=line value=`anchor` *
> subtask.spawn fn=persist_slot args=`wb`

> print text=spawned
