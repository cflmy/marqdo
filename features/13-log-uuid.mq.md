---
markup: occlusion
title: Mid stdlib — log and uuid
description: Level-filtered logs + UUID v4 (Mid M6)
import:
  log: lib/log.mq.md
  uuid: lib/uuid.mq.md
---

# Observable scripts without plugins


Default log level is info; raise to debug when you need more. Mint ids with uuid.v4.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We now **[Call log.info](> log.info text="mid-m6-start")**.

We now **[Call log.debug](> log.debug text="skipped-by-default")**.

We now **[Call log.set level](> log.set_level level="debug")**.

We now **[Call log.debug](> log.debug text="now-visible")**.

We first **id = [Compute id](uuid.v4())** for the next step.
We first **n = [Compute n](len(`id`))** for the next step.
We now **[Display this step's result](> print text=`n`)**.
