---
title: Mid stdlib — log and uuid
description: Level-filtered logs + UUID v4 (Mid M6)
import log:lib/log.mq.md
import uuid:lib/uuid.mq.md
---

# Observable scripts without plugins

Default log level is info; raise to debug when you need more. Mint ids with uuid.v4.

# main

> log.info text="mid-m6-start"

> log.debug text="skipped-by-default"

> log.set_level level="debug"

> log.debug text="now-visible"

*id = > uuid.v4*
*n = > len `id`*
> print text=`n`
