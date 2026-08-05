---
title: lib/sys — process
description: Env, cwd, args, exit, exec
> lib/sys.mq.md
---

# main

Import lib/sys.mq.md. Functions: env_get, env_set, args, cwd, exit(code), exec(cmd). Under view, exit soft-fails instead of killing the process.

*`d` = > cwd *

*`n` = > len `d` *

+ `n` > 0
  > print text=cwd-ok
+ *
  > print text=cwd-empty
