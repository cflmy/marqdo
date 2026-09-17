---
title: web tenant bag smoke
description: app.tenant mode=path stores config
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

**首页 = > web.page title="t"**
**应用 = > web.app page=`首页` admin=False**
**应用 = > `应用`.tenant mode="path" param="t" column="tenant_id" default_scope=True**
**bag = > json.get value=`应用` key="tenant"**
**mode = > json.get value=`bag` key="mode"**
**param = > json.get value=`bag` key="param"**
**scope = > json.get value=`bag` key="default_scope"**
1. `mode` == "path"
  1. `param` == "t"
    1. `scope` == True
      > print text=tenant-bag-ok
    2. *
      > print text=scope-fail
      > sys.exit code=1
  2. *
    > print text=param-fail
    > sys.exit code=1
2. *
  > print text=mode-fail
  > sys.exit code=1
