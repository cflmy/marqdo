---
title: lib/subtask — concurrent tasks
description: Spawn file, function, or foreign code in parallel
> lib/subtask.mq.md
---

# main

spawn picks one mode (others default to None):

| Argument | Runs | wait returns |
|----------|------|----------------|
| path= | marqdo run child process | exit code |
| fn= + optional args= | function in a thread (entry module) | return value |
| code= | foreign fence subprocess | stdout text |
| lang= + source= | foreign source subprocess | stdout text |

File children are silent unless quiet=False. Also: poll id=, kill id=, wait_all. Parent exit kills file/foreign children.

> print text=subtask overview ok
