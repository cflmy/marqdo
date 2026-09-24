---
title: Agent zero-LLM execution after policy/skill compile
description: After Skill+Policy compilation, run with router=marqdo records llm_calls=0.
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import llm:ext/ai/llm.mq.md
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

**paths = > json.parse text={"mem":".marqdo/agent-memory-zero-llm","kb":".marqdo/agent-kb-zero-llm","task":"Reply with exactly the word pong and nothing else.","skill":"concepts/skills/reply-with-exactly-the-word-pong-and-nothing-els.md"}**
**mem = > json.get value=`paths` key="mem"**
**kb = > json.get value=`paths` key="kb"**
**task = > json.get value=`paths` key="task"**
**skill = > json.get value=`paths` key="skill"**

**ex_m = > fs.exists path=`mem`**
1. `ex_m`
  > fs.remove_tree path=`mem`
2. *
  **_ = 1**
**ex_k = > fs.exists path=`kb`**
1. `ex_k`
  > fs.remove_tree path=`kb`
2. *
  **_ = 1**
> agent_memory_ensure memory_dir=`mem`

**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=2 tokens=100 memory_dir=`mem`**
**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=2 tokens=80 memory_dir=`mem`**
**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="explore" skill=`skill` llm_calls=1 tokens=40 memory_dir=`mem`**
**_ = > agent_maybe_learn task=`task` memory_dir=`mem` kb_dir=`kb` improve_every=3 promote=True**
**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 tokens=0 memory_dir=`mem`**
**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 tokens=0 memory_dir=`mem`**
**_ = > agent_record_episode task=`task` status="ok" result="pong" mode="compiled" skill=`skill` llm_calls=0 tokens=0 memory_dir=`mem`**
**_ = > agent_compile_policy memory_dir=`mem` min_evidence=3**

**model = > llm.create**
`tools` =

| 工具 |
|------|

**ag = > agent.agent model=`model` tools=`tools` standing="zero-llm compiled run"**
**out = > `ag`.run task=`task` kb_dir=`kb` memory_dir=`mem` learn=False router="marqdo" writeback=False**
**ex = > json.get value=`out` key="execution"**
**calls = > json.get value=`ex` key="llm_calls"**
**mode = > json.get value=`ex` key="mode"**
> print text=`mode`
> print text=`calls`
