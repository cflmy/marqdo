---
title: agent small-llm route request (offline)
description: router=small-llm returns needs_llm + prompt; jev alias same shape.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**r = > agent_route task="say hello" backend="small-llm" kb_dir=".marqdo/agent-kb-route-test" memory_dir=".marqdo/agent-memory-route-test"**
**needs = > json.get value=`r` key="needs_llm"**
**backend = > json.get value=`r` key="backend"**
**prompt = > json.get value=`r` key="prompt"**
1. `needs`
  1. `backend` == "small-llm"
    1. `prompt`
      > print text=small-llm-ok
    2. *
      > print text=small-llm-no-prompt
  2. *
    > print text=small-llm-backend-fail
2. *
  > print text=small-llm-fail

**j = > agent_route task="say hello" backend="jev" kb_dir=".marqdo/agent-kb-route-test" memory_dir=".marqdo/agent-memory-route-test"**
**jb = > json.get value=`j` key="backend"**
**jn = > json.get value=`j` key="needs_llm"**
1. `jn`
  1. `jb` == "small-llm"
    > print text=jev-alias-ok
  2. *
    > print text=jev-alias-backend-fail
    > print text=`jb`
2. *
  > print text=jev-alias-fail
