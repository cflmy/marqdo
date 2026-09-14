---
title: lib/subtask
description: Concurrent subtasks — file, function, or foreign code
---

Concurrent subtasks: file, function, or foreign code.

## spawn
    + `path`=None
    + `fn`=None
    + `args`=None
    + `code`=None
    + `lang`=None
    + `source`=None
    + `stdin`=None
    + `quiet`=True

Start a subtask. File children are silent by default (quiet=True: piped capture). Set quiet=False to inherit parent stdout/stderr. File wait returns {code, value} plus optional stdout/stderr tails when quiet-captured. value is the child main return (not stdout).

Caller: [subtask.spawn] path="child.mq.md"

*> host_subtask_spawn path=`path` fn=`fn` args=`args` code=`code` lang=`lang` source=`source` stdin=`stdin` quiet=`quiet`*

## poll
    + `id`

Non-blocking status for subtask id.

*> host_subtask_poll id=`id`*

## wait
    + `id`

Block until subtask id finishes.

*> host_subtask_join id=`id`*

## kill
    + `id`

Kill subtask id.

*> host_subtask_kill id=`id`*

## wait_all

Wait until every spawned subtask finishes.

*> host_subtask_wait_all*
