---
title: lib/subtask
description: OS subprocess subtasks (parent exit kills children)
---

## spawn
    + `path`

**> host_subtask_spawn path=`path`**

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
