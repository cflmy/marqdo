---
markup: occlusion
title: F1 eng_bind_execute + publish_gate (no LLM)
description: >-
  Deterministic fake-model substitute: bind+execute existing knowledge with
  actual_reuse trace; candidate_only / product_gate_ok=false refuse promote.
  Offline — no network / no LLM.
import:
  fs: lib/fs.mq.md
  json: lib/json.mq.md
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

# main

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**root = "tests/executable_knowledge/baseline/02-extract"**
`inputs` = > json.parse text={"数量":2,"单价":30}

**exec = > agent_eng_bind_execute project=`root` query="商品小计" policy="pure" inputs=`inputs` candidate_only=True**
**ok = > json.get value=`exec` key="ok"**
1. `ok`
  > print text=bind-execute-ok
2. *
  > print text=bind-execute-FAIL
  > print text=`exec`
  > sys.exit code=1

**reuse = > json.get value=`exec` key="actual_reuse"**
1. `reuse`
  > print text=actual-reuse-ok
2. *
  > print text=actual-reuse-MISSING
  > sys.exit code=1

**gate1 = > agent_eng_publish_gate project=`root` artifact_id="商品小计" candidate_only=True product_gate_ok=True**
**promote1 = > json.get value=`gate1` key="may_promote"**
1. `promote1` == False
  > print text=candidate-only-blocks-promote
2. *
  > print text=candidate-only-FAIL
  > print text=`gate1`
  > sys.exit code=1

**gate2 = > agent_eng_publish_gate project=`root` artifact_id="商品小计" candidate_only=False product_gate_ok=False**
**promote2 = > json.get value=`gate2` key="may_promote"**
1. `promote2` == False
  > print text=product-gate-blocks-promote
2. *
  > print text=product-gate-FAIL
  > print text=`gate2`
  > sys.exit code=1

> print text=agent-eng-bind-gate-ok
*"agent-eng-bind-gate-ok"*
