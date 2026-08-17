---
title: plugin demo ABI
description: Loads demo cdylib; path from MARQDO_TEST_PLUGIN (set by gold harness).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > sys.env_get name=MARQDO_TEST_PLUGIN*

> plugin.load path=`p`

*sum = > demo_add a=1 b=2*

> print text=`sum`

*echo = > demo_echo text=ok*

> print text=`echo`
