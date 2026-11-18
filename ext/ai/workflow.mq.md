---
markup: occlusion
title: Native AI workflow — durable, short, one-step-at-a-time execution
description: Plugin-managed checkpointed decision flow without an external agent runtime.
import:
  plugin: lib/plugin.mq.md
  json: lib/json.mq.md
  flow: ext/ai/flow.mq.md
---

Every step is an explicit commit boundary. The plugin stores accepted outputs,
refuses stale documents and conflicting decisions, and exposes the next action.
The caller must supply trusted session, principal and source revision fields;
model text is never an identity or permission token.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| steps | list | required | 1–128 ordered step records |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Created, resumed, or explicit conflict |

Use `id`, `steps`, `source_revision`, `session_id`=None, `principal_ref`=None as the inputs for this operation.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Open durable workflow](agent_workflow_open(id=`id`, steps=`steps`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## next

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Current action or terminal state |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None as the inputs for this operation.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Find the next committed boundary](agent_workflow_next(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## commit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| step_id | text | required | Exact current step |
| response | any | required | Completed step output |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Committed next step or explicit failure |

Use `id`, `step_id`, `response`, `source_revision`, `session_id`=None, `principal_ref`=None as the inputs for this operation.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Validate and commit one step](agent_workflow_commit(id=`id`, step_id=`step_id`, response=`response`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## cancel

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Cancelled checkpoint status |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None as the inputs for this operation.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Commit cancellation](agent_workflow_cancel(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.


## tick

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Running workflow |
| source_revision | text | required | Pinned source revision |
| registry | map | None | Trusted SKILL registry |
| model | any | None | Configured LLM |
| session_id | text | None | Host-attested session |
| principal_ref | text | None | Host-attested principal |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | New checkpoint state, or waiting_input for the UI |

Use `id`, `source_revision`, `registry`=None, `model`=None, `session_id`=None, `principal_ref`=None as the inputs for this operation.

Advance at most one step. Model text is a candidate, never direct
execution. Waiting for a user returns immediately without holding a thread.
A caller repeats tick after verified completion; UI replies use commit.

We retain **state = [Load the next action](next(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))**.
We retain **status = [Read action status](json.get(value=`state`, key="status"))**.
1. `status` == "waiting_model"
  We retain **step = [Read model step](json.get(value=`state`, key="step"))**.
  We retain **step_id = [Read step identifier](json.get(value=`step`, key="id"))**.
  We retain **prompt = [Read local prompt](json.get(value=`step`, key="prompt"))**.
  We retain **options = [Read allowed decisions](json.get(value=`step`, key="options"))**.
  We retain **answer = [Make a bounded choice](flow.choose(input=`prompt`, options=`options`, model=`model`))**.
  We retain **valid = [Check model decision](json.get(value=`answer`, key="ok"))**.
  1. `valid`
    We retain **chosen = [Read verified model answer](json.get(value=`answer`, key="value"))**.
    Return *[Commit the model choice](commit(id=`id`, step_id=`step_id`, response=`chosen`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
  2. *
    Return *answer*.
2. `status` == "ready_tool"
  We retain **step = [Read tool step](json.get(value=`state`, key="step"))**.
  We retain **kind = [Read operation kind](json.get(value=`step`, key="kind"))**.
  We retain **step_id = [Read step identifier](json.get(value=`step`, key="id"))**.
  1. `kind` == "parallel"
    We retain **tasks = [Read child tasks](json.get(value=`step`, key="tasks"))**.
    We retain **report = [Run bounded parallel documents](flow.parallel(tasks=`tasks`))**.
    Return *[Commit all child outcomes](commit(id=`id`, step_id=`step_id`, response=`report`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
  2. `kind` == "skill"
    We retain **name = [Read registered skill name](json.get(value=`step`, key="skill"))**.
    We retain **out = [Run registered SKILL only](flow.enter(skill=`name`, registry=`registry`))**.
    Return *[Commit verified child result](commit(id=`id`, step_id=`step_id`, response=`out`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
  3. *
    Return *state*.
3. `status` == "ready_return"
  We retain **step = [Read return step](json.get(value=`state`, key="step"))**.
  We retain **step_id = [Read step identifier](json.get(value=`step`, key="id"))**.
  We retain **value = [Read final value](json.get(value=`step`, key="value"))**.
  Return *[Commit the completed workflow](commit(id=`id`, step_id=`step_id`, response=`value`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`))*.
4. *
  Return *state*.
