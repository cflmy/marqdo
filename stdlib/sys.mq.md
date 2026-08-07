---
title: lib/sys — process
description: Env, dotenv, cwd, args, exit, exec
> lib/sys.mq.md
---

# main

Import lib/sys.mq.md. Functions: env_get, env_set, load_dotenv(optional path), args, cwd, exit(code), exec(cmd). load_dotenv does not override existing variables. Under view, exit soft-fails instead of killing the process.

*`d` = > cwd *

*`n` = > len `d` *

1. `n` > 0
  > print text=cwd-ok
2. *
  > print text=cwd-empty
