---
title: plugin demo ABI
description: Loads demo cdylib; path from MARQDO_TEST_PLUGIN (set by gold harness).
> lib/plugin.mq.md
> lib/sys.mq.md
---

# main

*`p` = > env_get name=MARQDO_TEST_PLUGIN *

> load path=`p`

*`sum` = > demo_add a=1 b=2 *

> print text=`sum`

*`echo` = > demo_echo text=ok *

> print text=`echo`
