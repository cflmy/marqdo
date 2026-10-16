---
title: agent.run cold OKF — resolve llm_free + self.preflight
description: >-
  Regression for qdagent-facing bugs (2026-10):
  (A) miss resolve must expose llm_free=false;
  (B) `self`.preflight must resolve when agent is imported by a consumer.
  Offline — dummy API key; no live network required for assertions A/B.
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import llm:ext/ai/llm.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

> sys.env_set name="OPENAI_API_KEY" value="offline-cold-efi-dummy"

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**kb = ".marqdo/agent-kb-cold-efi"**
**ex = > fs.exists path=`kb`**
1. `ex`
  > fs.remove_tree path=`kb`
2. *
  **_ = 1**
> fs.make_dir path=`kb`

**resolved = > agent_resolve task="qdagent-cold-unknown-task-xyz-20261016" kb_dir=`kb`**
**lf = > json.get value=`resolved` key="llm_free"**
**mode = > json.get value=`resolved` key="mode"**
1. `lf` == False
  > print text=resolve-miss-llm_free-ok
2. *
  > print text=resolve-miss-llm_free-MISSING
  > print text=`resolved`
  > sys.exit code=1

1. `mode` == "unknown"
  > print text=resolve-miss-mode-ok
2. *
  > print text=resolve-miss-mode-unexpected
  > sys.exit code=1

**model = > llm.llm**
`tools` =

| 工具 |
|------|

**ag = > agent.agent model=`model` tools=`tools` standing="cold efi preflight probe"**
**pf = > `ag`.preflight task="qdagent-cold-unknown-task-xyz-20261016" marqdo_dir=".marqdo"**
**st = > json.get value=`pf` key="status"**
1. `st`
  > print text=self-preflight-ok
  > print text=`st`
2. *
  > print text=self-preflight-failed
  > print text=`pf`
  > sys.exit code=1

**via = > `ag`.resolve task="qdagent-cold-unknown-task-xyz-20261016" kb_dir=`kb`**
**via_lf = > json.get value=`via` key="llm_free"**
1. `via_lf` == False
  > print text=method-resolve-llm_free-ok
2. *
  > print text=method-resolve-llm_free-MISSING
  > sys.exit code=1

*"agent-run-cold-efi-ok"*
