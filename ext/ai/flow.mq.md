---
markup: occlusion
title: AI flow — small-model choices and bounded subtasks
description: Short model decisions, verified options and concurrent Marqdo documents.
import:
  llm: ext/ai/llm.mq.md
  json: lib/json.mq.md
  plugin: lib/plugin.mq.md
  subtask: lib/subtask.mq.md
---

This is the thin author-facing layer. A model returns a candidate; Marqdo checks
it before routing. Invalid choices are not SKILL paths and never execute.
Only the requested function and its inputs need to be in the model prompt.

## parse_choice

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| reply | text | required | Raw model reply, treated as data |
| options | list | required | Allowed names, not paths or commands |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | {ok,value,status} or {ok:false,error} |

Parsing accepts one exact name, a labeled name, a fenced name or a 1-based
index. Ambiguous prose fails closed. Parsing never runs a tool or SKILL.

We retain **p = [Locate the installed agent plugin](plugin.native_path(name="agent"))**.
1. p
  We **[Load the agent plugin](plugin.load(path=`p`))**.
2. *
  Return *[Missing native parser](json.parse(text="{\"ok\":false,\"status\":\"unavailable\",\"value\":null}"))*.

Return *[Parse and verify the choice](agent_choice_parse(reply=`reply`, options=`options`))*.

## choose

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| input | text | required | Current user request or local decision |
| options | list | required | Small list of registered option names |
| model | any | None | Existing provider model identifier |
| repair | bool | True | Allow one format-only repair attempt |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Verified choice result, or explicit failure |

Only ask for the option name. A failed first parse can receive one short
format correction; never silently select the nearest-looking option.

We retain **opts = [Encode allowed values](json.stringify(value=`options`))**.
We retain **prompt = "Choose exactly one of these options. Return its name only, no explanation:\n" + opts + "\nInput:\n" + input**.
We retain **reply = [Ask the model once](llm.ask(prompt=`prompt`, model=`model`))**.
We retain **parsed = [Validate the model reply](parse_choice(reply=`reply`, options=`options`))**.
We retain **ok = [Read validation status](json.get(value=`parsed`, key="ok"))**.
1. ok
  Return *parsed*.
2. `repair`
  We retain **fix = "Your answer was not an exact allowed option. Return only ONE option name from:\n" + opts**.
  We retain **again = [Retry the format only once](llm.ask(prompt=`fix`, model=`model`))**.
  Return *[Validate the repaired reply](parse_choice(reply=`again`, options=`options`))*.
3. *
  Return *parsed*.

## parallel

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| tasks | list | required | Existing .mq.md file tasks with arguments |
| limit | number | 2 | Maximum simultaneously running tasks |
| timeout_ms | number | 30000 | Execution timeout in milliseconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Ordered child outcomes, including failures |

Reuse the built-in subtask scheduler. Each child document returns via its
# main, and successful siblings remain visible if another child fails.

Return *[Run bounded Marqdo document tasks](subtask.run_many(tasks=`tasks`, limit=`limit`, timeout_ms=`timeout_ms`))*.
