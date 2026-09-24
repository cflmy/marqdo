---
title: Agent Framework v2 — episode → learn → compile (offline)
description: Record three identical successes, maybe_learn compiles llm_free skill, resolve hits compiled path.
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**paths = > json.parse text={"mem":".marqdo/agent-memory-v2-test","kb":".marqdo/agent-kb-v2-test","task":"Reply with exactly the word pong and nothing else."}**
**mem = > json.get value=`paths` key="mem"**
**kb = > json.get value=`paths` key="kb"**
**task = > json.get value=`paths` key="task"**

> agent_memory_ensure memory_dir=`mem`

**ep1 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" llm_calls=2 memory_dir=`mem`**
**ep2 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" llm_calls=2 memory_dir=`mem`**
**ep3 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" llm_calls=2 memory_dir=`mem`**
**ok = > json.get value=`ep3` key="ok"**
1. `ok`
  > print text=ep-ok
2. *
  > print text=ep-fail

**listed = > agent_list_episodes memory_dir=`mem` task=`task`**
**n = > json.get value=`listed` key="count"**
> print text=`n`

**learned = > agent_maybe_learn task=`task` memory_dir=`mem` kb_dir=`kb` improve_every=3 promote=True**
**did = > json.get value=`learned` key="learned"**
1. `did`
  > print text=learned
2. *
  > print text=learn-skip

**resolved = > agent_resolve task=`task` kb_dir=`kb`**
**lf = > json.get value=`resolved` key="llm_free"**
1. `lf`
  > print text=llm-free
2. *
  > print text=not-free

**m = > agent_metrics memory_dir=`mem`**
**eps = > json.get value=`m` key="episodes"**
> print text=`eps`
