---
markup: occlusion
title: lib/subtask — concurrent tasks
description: Spawn file, function, or foreign code in parallel
import:
  subtask: lib/subtask.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

spawn picks one mode (others default to None).

path= runs a marqdo run child process; wait returns the exit code.

fn= plus optional args= runs a function in a thread; wait returns the return value.

code= runs a foreign fence subprocess; wait returns stdout text.

lang= plus source= runs foreign source; wait returns stdout text.

File children are silent unless quiet=False. Also: poll id=, kill id=, wait_all. Parent exit kills file/foreign children.

We now **[Display this step's result](> print text="subtask overview ok")**.
