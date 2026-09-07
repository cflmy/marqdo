---
title: web db cross-module smoke
description: Return a web.db handle from an imported module and call select in the entry module.
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import storemod:web-db-cross-module-lib.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

*store = > storemod.open*
*rows = > `store`.select table="items" limit=10*
*n = > len value=`rows`*
1. `n` >= 1
  > print text=cross-db-ok
2. *
  > print text=cross-db-fail
