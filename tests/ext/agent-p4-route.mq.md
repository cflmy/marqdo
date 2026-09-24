---
title: Agent Framework P4 — Adaptive Routing + Policy Compilation (offline)
description: Learn skill, compile policy from skill-tagged episodes, route via marqdo backend without Jev/LLM.
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

**paths = > json.parse text={"mem":".marqdo/agent-memory-p4-test","kb":".marqdo/agent-kb-p4-test","task":"Reply with exactly the word pong and nothing else.","skill":"concepts/skills/reply-with-exactly-the-word-pong-and-nothing-els.md"}**
**mem = > json.get value=`paths` key="mem"**
**kb = > json.get value=`paths` key="kb"**
**task = > json.get value=`paths` key="task"**
**skill = > json.get value=`paths` key="skill"**

> agent_memory_ensure memory_dir=`mem`

**ep1 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=2 memory_dir=`mem`**
**ep2 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=2 memory_dir=`mem`**
**ep3 = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=2 memory_dir=`mem`**

**learned = > agent_maybe_learn task=`task` memory_dir=`mem` kb_dir=`kb` improve_every=3 promote=True**
**did = > json.get value=`learned` key="learned"**
1. `did`
  > print text=skill-learned
2. *
  > print text=skill-skip

**ep4 = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 memory_dir=`mem`**
**ep5 = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 memory_dir=`mem`**
**ep6 = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 memory_dir=`mem`**

**pol = > agent_compile_policy memory_dir=`mem` min_evidence=3**
**pok = > json.get value=`pol` key="ok"**
1. `pok`
  > print text=policy-ok
2. *
  > print text=policy-fail

**routed = > agent_route task=`task` backend="auto" memory_dir=`mem` kb_dir=`kb`**
**matched = > json.get value=`routed` key="matched"**
**needs = > json.get value=`routed` key="needs_llm"**
1. `matched`
  1. not `needs`
    > print text=route-hit
  2. *
    > print text=route-needs-llm
2. *
  > print text=route-miss

**backend = > json.get value=`routed` key="backend"**
> print text=`backend`
