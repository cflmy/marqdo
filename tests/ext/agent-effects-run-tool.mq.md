---
markup: occlusion
title: agent-effects-run-tool
description: >-
  M-A5a / MA-T03 subset — mutating run_tool goes through EffectLedger;
  committed effects dedupe; started-without-commit becomes unknown (no re-exec).
import:
  agent: /home/cflmy/work/marqdo/ext/ai/agent.mq.md
  json: lib/json.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

## write_probe

Probe side-effect tool body (count stays in ledger, not here).

*"wrote"*

# main

**schema = > json.parse text={"type":"object","properties":{},"additionalProperties":true}**
**scopes = > json.parse text=["workspace:write"]**
`spec` =

| id | version | description | input_schema | effect | scopes | availability |
|----|---------|-------------|--------------|--------|--------|--------------|
| write_probe | 1 | test write tool | `schema` | write | `scopes` | ready |

> agent.tools_clear
**reg = > agent.tools_register spec=`spec`**
**allow = > json.parse text=["write_probe"]**
**args = > json.parse text={}**

**out1 = > agent.run_tool tools=`allow` name="write_probe" arguments=`args` logical_run_id="run_ma5a" step_id="s1" call_id="c1" operation_id="op_write_probe_c1" ledger_dir=".marqdo/agent-effects-ma5a"**
**ok1 = > json.get value=`out1` key="ok"**
**st1 = > json.get value=`out1` key="status"**
**rc1 = > json.get value=`out1` key="effect_receipt_ref"**
> print text=`ok1`
> print text=`st1`
> print text=`rc1`

**out2 = > agent.run_tool tools=`allow` name="write_probe" arguments=`args` logical_run_id="run_ma5a" step_id="s1" call_id="c1" operation_id="op_write_probe_c1" ledger_dir=".marqdo/agent-effects-ma5a"**
**sum2 = > json.get value=`out2` key="summary"**
> print text=`sum2`

**wm = > agent.effects_watermark ledger_dir=".marqdo/agent-effects-ma5a" logical_run_id="run_ma5a"**
**safe = > json.get value=`wm` key="write_auto_resume_safe"**
> print text=`safe`

**got = > agent.effects_get operation_id="op_write_probe_c1" ledger_dir=".marqdo/agent-effects-ma5a"**
**rec = > json.get value=`got` key="record"**
**state = > json.get value=`rec` key="state"**
> print text=`state`
