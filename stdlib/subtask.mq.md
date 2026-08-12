---
title: lib/subtask — concurrent tasks
description: Spawn file, function, or foreign code in parallel
> lib/subtask.mq.md
---

# main

spawn picks one mode (others default to None).

path= runs a marqdo run child process; wait returns the exit code.

fn= plus optional args= runs a function in a thread; wait returns the return value.

code= runs a foreign fence subprocess; wait returns stdout text.

lang= plus source= runs foreign source; wait returns stdout text.

File children are silent unless quiet=False. Also: poll id=, kill id=, wait_all. Parent exit kills file/foreign children.

> print text=subtask overview ok
