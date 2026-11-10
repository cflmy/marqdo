---
markup: occlusion
title: lib/sys — process
description: Env, dotenv, cwd, args, exit, exec
import:
  sys: lib/sys.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Import lib/sys.mq.md. Functions: env_get, env_set, load_dotenv(optional path), args, cwd, exit(code), exec(cmd), stream_publish(event). load_dotenv does not override existing variables. Under view, exit soft-fails instead of killing the process. `stream_publish` feeds the view SSE EventBus.

We first **d = [Compute d](sys.cwd())** for the next step.

We first **n = [Compute n](len(`d`))** for the next step.

1. `n` > 0
  We now **[Display this step's result](> print text="cwd-ok")**.
2. *
  We now **[Display this step's result](> print text="cwd-empty")**.
