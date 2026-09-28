---
title: eng preflight offline
import plugin:lib/plugin.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
---

Offline Engineering Knowledge gate against `tests/engineering/.marqdo`
(seed with `marqdo knowledge tests/engineering -o tests/engineering/.marqdo`).

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  **out = "skip"**
  *out*

**pf = > agent_eng_preflight task="resolve configuration" marqdo_dir="tests/engineering/.marqdo"**
**status = > json.get value=`pf` key="status"**
**dec = > json.get value=`pf` key="decision"**
**out = "eng-preflight-unexpected"**
1. `status` == "missing_knowledge"
  **out = "eng-preflight-skip-no-graph"**
2. `dec` == "REUSE"
  **out = "eng-preflight-ok"**
3. `dec` == "ADAPT"
  **out = "eng-preflight-ok"**
4. *
  **out = "eng-preflight-unexpected"**

*out*
