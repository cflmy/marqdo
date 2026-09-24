---
title: agent resume checkpoint (offline)
description: save / load / list / clear under .marqdo/agent-resume-test
import agent:ext/ai/agent.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**dir = ".marqdo/agent-resume-test"**
**exists = > fs.exists path=`dir`**
1. `exists`
  > fs.remove_tree path=`dir`
2. *
  **_ = 1**

**saved = > agent.resume_save id="demo1" goal="ping" status="running" round=2 resume_dir=`dir`**
**ok = > json.get value=`saved` key="ok"**
1. `ok`
  > print text=resume-save-ok
2. *
  > print text=resume-save-fail

**loaded = > agent.resume id="demo1" resume_dir=`dir`**
**found = > json.get value=`loaded` key="found"**
**round = > json.get value=`loaded` key="round"**
1. `found`
  1. `round` == 2
    > print text=resume-load-ok
  2. *
    > print text=resume-round-fail
2. *
  > print text=resume-load-fail

**listed = > agent.resume_list resume_dir=`dir`**
**items = > json.get value=`listed` key="items"**
**n = > len value=`items`**
1. `n` == 1
  > print text=resume-list-ok
2. *
  > print text=resume-list-fail

**cleared = > agent.resume_clear id="demo1" resume_dir=`dir`**
**c = > json.get value=`cleared` key="cleared"**
1. `c`
  > print text=resume-clear-ok
2. *
  > print text=resume-clear-fail

**loaded2 = > agent.resume_load id="demo1" resume_dir=`dir`**
**found2 = > json.get value=`loaded2` key="found"**
1. not `found2`
  > print text=resume-gone-ok
2. *
  > print text=resume-gone-fail
