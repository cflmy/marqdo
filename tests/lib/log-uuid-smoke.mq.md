---
title: lib/log + uuid smoke
import log:lib/log.mq.md
import uuid:lib/uuid.mq.md
import re:lib/re.mq.md
---

# main

> log.info text="hello-info"

> log.debug text="hidden"

> log.set_level level="debug"

> log.debug text="shown-debug"

*id = > uuid.v4*
*n = > len `id`*
> print text=`n`
*ok = > re.is_match text=`id` pattern="^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"*
> print text=`ok`
