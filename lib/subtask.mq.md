---
title: lib/subtask
description: Concurrent subtasks — file, function, or foreign code
---

## spawn
    + `path`=None
    + `fn`=None
    + `args`=None
    + `code`=None
    + `lang`=None
    + `source`=None
    + `stdin`=None

**> host_subtask_spawn path=`path` fn=`fn` args=`args` code=`code` lang=`lang` source=`source` stdin=`stdin`**

## poll
    + `id`

**> host_subtask_poll id=`id`**

## wait
    + `id`

**> host_subtask_join id=`id`**

## kill
    + `id`

**> host_subtask_kill id=`id`**

## wait_all

**> host_subtask_wait_all**
