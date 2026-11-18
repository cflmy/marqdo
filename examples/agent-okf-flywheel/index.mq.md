---
markup: occlusion
title: agent-okf-flywheel (Wave B0)
description: Promote solidified workbook; second plan is cache=hit — document as knowledge base (OKF).
import:
  llm: ext/ai/llm.mq.md
  agent: ext/ai/agent.mq.md
  fs: lib/fs.mq.md
  json: lib/json.mq.md
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

This is an offline knowledge-reuse workflow, not a request to an external model. It loads the agent plugin, promotes the visible workbook for one exact goal and asks the same agent goal again. The expected hit comes from the promoted knowledge base. The dummy key only satisfies model setup; it is not a credential or evidence that a model call succeeded.

The example uses its own named files under .marqdo and replaces that example knowledge base on rerun. Grant the actual plugin and filesystem effects in a disposable example root. Promotion and execution are separate outcomes: inspect promoted first, then cache, match and the returned result. A hit marker alone does not prove that an unrelated new goal is safe to reuse.

OKF flywheel offline: promote → plan same goal → `cache=hit` (no live LLM required for the hit path).

We first **p = [Compute p](plugin.native_path(name="agent"))** for the next step.
1. `p`
  We now **[Call plugin.load](> plugin.load path=`p`)**.
2. *
  We now **[Display this step's result](> print text="no-agent-plugin")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We now **[Call sys.env set](> sys.env_set name="OPENAI_API_KEY" value="offline-okf-dummy")**.
We first **model = [Compute model](llm.llm())** for the next step.
We first **tools = [Compute tools](json.parse(text="[]"))** for the next step.
We first **助手 = [Compute 助手](agent.agent(model=`model`, tools=`tools`, standing="offline okf flywheel"))** for the next step.

We first **kb = ".marqdo/agent-kb-example-flywheel"** for the next step.
We first **wb = ".marqdo/agent-runs/example-pong-solid.mq.md"** for the next step.
We first **goal = "Reply with exactly the word pong and nothing else."** for the next step.
We first **body = "---\ntitle: okf pong\n---\n\n# main\n\n*msg = \"pong\"*\n\n**msg**\n"** for the next step.

1. > fs.exists path=`kb`
  We now **[Call fs.remove](> fs.remove path=`kb`)**.
2. *
  ---

We now **[Call fs.write text](> fs.write_text path=`wb` text=`body`)**.

We first **prom = [Compute prom](agent_kb_promote(kb_dir=`kb`, goal=`goal`, workbook=`wb`))** for the next step.
We first **okp = [Compute okp](json.get(value=`prom`, key="promoted"))** for the next step.
1. `okp`
  We now **[Display this step's result](> print text="promoted")**.
2. *
  We now **[Display this step's result](> print text="promote-fail")**.
  We now **[Call sys.exit](> sys.exit code=1)**.

We first **out = [Compute out](> `助手`.plan goal=`goal` max_rounds=2 writeback=False kb_dir=`kb` explore_n=0 promote=False)** for the next step.
We first **cache = [Compute cache](json.get(value=`out`, key="cache"))** for the next step.
We now **[Display this step's result](> print text=`cache`)**.
We first **match = [Compute match](json.get(value=`out`, key="match"))** for the next step.
We now **[Display this step's result](> print text=`match`)**.
We first **val = [Compute val](json.get(value=`out`, key="result"))** for the next step.
We now **[Display this step's result](> print text=`val`)**.

1. `cache` == "hit"
  We now **[Display this step's result](> print text="flywheel-ok")**.
2. *
  We now **[Display this step's result](> print text="flywheel-miss")**.
  We now **[Call sys.exit](> sys.exit code=1)**.
