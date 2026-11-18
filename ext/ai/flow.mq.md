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

Use `reply`, `options` as the inputs for this operation.

Parsing accepts one exact name, a labeled name, a fenced name or a 1-based
index. Ambiguous prose fails closed. Parsing never runs a tool or SKILL.

We retain **p = [Locate the installed agent plugin](plugin.native_path(name="agent"))**.
1. `p`
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

Use `input`, `options`, `model`=None, `repair`=True as the inputs for this operation.

Only ask for the option name. A failed first parse can receive one short
format correction; never silently select the nearest-looking option.

We retain **opts = [Encode allowed values](json.stringify(value=`options`))**.
We retain **prompt = "Choose exactly one of these options. Return its name only, no explanation:\n" + opts + "\nInput:\n" + input**.
We retain **m = [Build one model handle](llm.create(model=`model`))**.
We retain **turn = [Ask the model once](m.ask(prompt=`prompt`))**.
We retain **err = [Check model transport error](json.get(value=`turn`, key="error"))**.
1. `err`
  Return *turn*.
2. *
  We retain **reply = [Read model answer](json.get(value=`turn`, key="text"))**.
  We retain **parsed = [Validate the model reply](parse_choice(reply=`reply`, options=`options`))**.
  We retain **ok = [Read validation status](json.get(value=`parsed`, key="ok"))**.
  1. `ok`
    Return *parsed*.
  2. `repair`
    We retain **fix = "Your answer was not an exact allowed option. Return only ONE option name from:\n" + opts**.
    We retain **fixed = [Retry the format only once](m.ask(prompt=`fix`))**.
    We retain **repair_error = [Read transport error](json.get(value=`fixed`, key="error"))**.
    1. `repair_error`
      Return *fixed*.
    2. *
      We retain **again = [Read corrected answer](json.get(value=`fixed`, key="text"))**.
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
| result | map | Succeeded, partial, failed or uncertain child outcomes |

Use `tasks`, `limit`=2, `timeout_ms`=30000 as the inputs for this operation.

Reuse the built-in subtask scheduler. Each child document returns via its
`# main`, and successful siblings remain visible if another child fails.

We retain **results = [Run bounded Marqdo document tasks](subtask.run_many(tasks=`tasks`, limit=`limit`, timeout_ms=`timeout_ms`))**.
We retain **p = [Locate native agent plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load native agent plugin](plugin.load(path=`p`))**.
  Return *[Preserve success, failure and uncertain child outcomes](agent_parallel_report(results=`results`))*.
2. *
  Return *[Native result classifier unavailable](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.


## input

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable step ID |
| question | text | required | User-visible question |
| options | list | required | User-visible choices |
| source_revision | text | required | Pinned Marqdo source revision |
| session_id | text | None | Host-attested session |
| principal_ref | text | None | Host-attested user scope |
| freeform | bool | True | Allow custom user reply |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | waiting_input or previously committed reply |

Use `id`, `question`, `options`, `source_revision`, `session_id`=None, `principal_ref`=None, `freeform`=True as the inputs for this operation.

This does not park an interpreter stack. The caller persists a step-boundary
checkpoint, displays the question, and on a new turn uses reply to continue.
The host, not the LLM, must attest session and principal IDs.

We retain **p = [Locate native agent plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load native agent plugin](plugin.load(path=`p`))**.
2. *
  Return *[No native gate available](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

Return *[Persist one waiting input gate](agent_input_open(id=`id`, question=`question`, options=`options`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, freeform=`freeform`))*.

## reply

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Existing waiting step |
| reply | text | required | New user response |
| source_revision | text | required | Current pinned source revision |
| session_id | text | None | Host-attested session |
| principal_ref | text | None | Host-attested user scope |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Resolved answer, invalid reply or explicit conflict |

Use `id`, `reply`, `source_revision`, `session_id`=None, `principal_ref`=None as the inputs for this operation.

Matching repeated replies are idempotent. A conflicting reply, changed document
or changed session fails without executing any SKILL or side effect.

We retain **p = [Locate native agent plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load native agent plugin](plugin.load(path=`p`))**.
2. *
  Return *[No native gate available](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

Return *[Commit the verified user reply](agent_input_resume(id=`id`, reply=`reply`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.


## enter

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| skill | text | required | Model-selected registered name |
| registry | map | required | Trusted name → path mapping |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Subtask result, or blocked unknown skill |

Use `skill`, `registry` as the inputs for this operation.

Never execute a raw model-generated path. Resolve a registered skill name to
an author-controlled document, then call its `# main` through the normal Marqdo
subtask and effect permissions. The caller must provide a trusted registry.

We retain **path = [Resolve registered SKILL](json.get(value=`registry`, key=`skill`))**.
1. not `path`
  Return *[Unknown skill is blocked](json.parse(text="{\"ok\":false,\"status\":\"unsupported_skill\"}"))*.
2. *
  We retain **task = [Spawn registered Marqdo document](subtask.spawn(path=`path`))**.
  Return *[Await the subtask result](subtask.wait(id=`task`))*.


## ask

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prompt | text | required | Short local prompt |
| model | any | None | Configured model selection |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Model reply (not yet a verified tool call) |

Use `prompt`, `model`=None as the inputs for this operation.

Ask only what the current function needs. The reply remains data until an
explicit verifier accepts it. The LLM adapter owns provider configuration.

Return *[Ask the configured model](llm.ask(prompt=`prompt`, model=`model`))*.

## start

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Trusted registered Marqdo document |
| args | list | None | Optional child CLI arguments |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Child task handle |

Use `path`, `args`=None as the inputs for this operation.

Start an isolated SKILL document asynchronously, after the calling host has
validated its path and required process.exec permission. No model-generated
raw path is a valid grant.

Return *[Start one Marqdo document](subtask.spawn(path=`path`, args=`args`))*.

## wait

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | number | required | Child task handle |
| timeout_ms | number | 30000 | Maximum wait time |
| cancel | bool | False | Cancel on timeout |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Child result, timeout or cancellation |

Use `id`, `timeout_ms`=30000, `cancel`=False as the inputs for this operation.

A successful wait protocol is not proof of a successful child execution:
inspect the child exit code before using its result.

Return *[Wait for the child](subtask.wait_timeout(id=`id`, timeout_ms=`timeout_ms`, cancel=`cancel`))*.
