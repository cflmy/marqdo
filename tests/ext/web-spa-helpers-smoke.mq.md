---
title: browser spa helpers smoke
description: store_set / spa_goto effect bags
import browser:lib/browser.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

**set = > browser.store_set key="k" value="v" scope="memory"**
**op = > json.get value=`set` key="storage"**
**op_name = > json.get value=`op` key="op"**
**scope = > json.get value=`op` key="scope"**
1. `op_name` == "set"
  1. `scope` == "memory"
    > print text=store-set-ok
  2. *
    > print text=scope-fail
    > sys.exit code=1
2. *
  > print text=op-fail
  > sys.exit code=1

**nav = > browser.spa_goto url="/item" replace=False**
**n = > json.get value=`nav` key="navigate"**
**url = > json.get value=`n` key="url"**
1. `url` == "/item"
  > print text=spa-goto-ok
2. *
  > print text=spa-goto-fail
  > sys.exit code=1

> print text=spa-helpers-ok
