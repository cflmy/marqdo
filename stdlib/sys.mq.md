---
title: lib/sys — process
description: Env, dotenv, cwd, args, exit, exec
import sys:lib/sys.mq.md
---

# main

Import lib/sys.mq.md. Functions: env_get, env_set, load_dotenv(optional path), args, cwd, exit(code), exec(cmd), stream_publish(event). load_dotenv does not override existing variables. Under view, exit soft-fails instead of killing the process. `stream_publish` feeds the view SSE EventBus.

*`d` = > sys.cwd *

*`n` = > len `d` *

1. `n` > 0
  > print text=cwd-ok
2. *
  > print text=cwd-empty
