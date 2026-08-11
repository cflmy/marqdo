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
    + `quiet`=True

File children are silent by default (`quiet=True`: piped capture, no TTY noise).
Set `quiet=False` to inherit parent stdout/stderr.
File `wait` returns `{code, value}` plus optional `stdout`/`stderr` tails when quiet-captured.
`value` is the child's `# main` return (not stdout).

**> host_subtask_spawn path=`path` fn=`fn` args=`args` code=`code` lang=`lang` source=`source` stdin=`stdin` quiet=`quiet`**

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
