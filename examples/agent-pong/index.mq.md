---
markup: occlusion
title: agent-pong (Wave B0)
description: >-
  Constitution demo — call_site + source in step context; optional live CALL via .env.
  Code is documentation; tools are ## in this same file.
import:
  llm: ext/ai/llm.mq.md
  agent: ext/ai/agent.mq.md
  json: lib/json.mq.md
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  time: lib/time.mq.md
---

## 获取时间

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

Return today's date string (tool visible in source to the model).

We first **u = [Compute u](time.now_unix())** for the next step.
After these steps, return *time.format(unix=`u`, pattern="%Y-%m-%d")*.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

This entry first verifies what the agent can read about its own call site. It assembles a step context and checks for the call-site section, source section and the actual time-tool name before considering an optional live step. The tool table points to the real local function; it is not an independently maintained tool description detached from the source.

Normal demonstration stops at context evidence and does not require a paid model response. AGENT_LIVE and an available key together opt into the separate live branch. Context markers establish prompt content, not the correctness of a future model answer. Keep the tool result and returned status visible when exercising that branch; never replace a missing credential with a fixed successful response.

We first **p = [Compute p](plugin.native_path(name="agent"))** for the next step.
1. `p`
  We now **[Call plugin.load](> plugin.load path=`p`)**.
2. *
  We now **[Display this step's result](> print text="no-agent-plugin")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We now **[Call llm.load env](> llm.load_env path="../../.env")**.

`工具表` =

| 工具 |
|------|
| 获取时间 |

We first **model = [Compute model](llm.llm())** for the next step.
We first **助手 = [Compute 助手](agent.agent(model=`model`, tools=`工具表`, standing="You are a Marqdo agent. Prefer CALL:获取时间 when the task needs today's date."))** for the next step.

Assert constitution markers in the assembled prompt (call site before how-to-act):

We first **ctx = [Compute ctx](agent.build_step_context(agent=`助手`, task="What is today's date? Use the tool in this runbook."))** for the next step.

We first **parts = [Compute parts](split(value=`ctx`, sep="--- call site ---"))** for the next step.
We first **n = [Compute n](len(value=`parts`))** for the next step.
1. `n` > 1
  We now **[Display this step's result](> print text="call-site-ok")**.
2. *
  We now **[Display this step's result](> print text="call-site-missing")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We first **parts2 = [Compute parts2](split(value=`ctx`, sep="--- source (.mq.md) ---"))** for the next step.
We first **n2 = [Compute n2](len(value=`parts2`))** for the next step.
1. `n2` > 1
  We now **[Display this step's result](> print text="source-ok")**.
2. *
  We now **[Display this step's result](> print text="source-missing")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We first **parts3 = [Compute parts3](split(value=`ctx`, sep="获取时间"))** for the next step.
We first **n3 = [Compute n3](len(value=`parts3`))** for the next step.
1. `n3` > 1
  We now **[Display this step's result](> print text="tool-in-source-ok")**.
2. *
  We now **[Display this step's result](> print text="tool-in-source-missing")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We first **site = [Compute site](agent_call_site())** for the next step.
We first **fn = [Compute fn](json.get(value=`site`, key="function"))** for the next step.
We now **[Display this step's result](> print text=`fn`)**.

Optional live step — set env AGENT_LIVE=1 and put keys in repo-root `.env`:

We first **live = [Compute live](sys.env_get(name="AGENT_LIVE"))** for the next step.
We first **key = [Compute key](sys.env_get(name="OPENAI_API_KEY"))** for the next step.
1. `live` == "1"
  1. `key`
    We first **out = [Compute out](> `助手`.step task="请调用获取时间工具，然后只回复日期字符串。" writeback=False)** for the next step.
    We first **status = [Compute status](json.get(value=`out`, key="status"))** for the next step.
    We now **[Display this step's result](> print text=`status`)**.
    We first **tool = [Compute tool](json.get(value=`out`, key="tool"))** for the next step.
    We now **[Display this step's result](> print text=`tool`)**.
    We first **result = [Compute result](json.get(value=`out`, key="result"))** for the next step.
    We now **[Display this step's result](> print text=`result`)**.
  2. *
    We now **[Display this step's result](> print text="live-skipped-no-key")**.
2. *
  We now **[Display this step's result](> print text="live-skipped")**.
