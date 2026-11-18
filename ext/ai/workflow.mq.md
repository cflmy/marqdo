---
markup: occlusion
title: Native AI workflow — durable, short, one-step-at-a-time execution
description: Plugin-managed checkpointed decision flow without an external agent runtime.
import:
  plugin: lib/plugin.mq.md
  json: lib/json.mq.md
  flow: ext/ai/flow.mq.md
  subtask: lib/subtask.mq.md
---

Every step is an explicit commit boundary. The plugin stores accepted outputs,
refuses stale documents and conflicting decisions, and exposes the next action.
The caller must supply trusted session, principal and source revision fields;
model text is never an identity or permission token. `resume_dir` is a
host-relative checkpoint directory, never an absolute client path.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| steps | list | required | 1–128 ordered step records |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Created, resumed, or explicit conflict |

Use `id`, `steps`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Opening the same ID with the same steps, source revision, session and principal
returns the current action instead of resetting progress. Reusing an ID with a
different plan or identity fails closed.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Open durable workflow](agent_workflow_open(id=`id`, steps=`steps`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## next

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Current action or terminal state |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

`next` is read-only: it does not call a model, run a child document or consume
the current step. Stopped states (`blocked`, `uncertain`, `cancelled`) and
unclassified child effects are reported explicitly.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Find the next committed boundary](agent_workflow_next(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## inspect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Progress summary with cursor, step and partial results |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Use inspect for a UI progress panel or an operator audit. It returns the
durable checkpoint `status`, the next `action` (for example `waiting_model`),
cursor, total and completed step counts, the current step, and retained partial
child results without advancing the workflow.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Read workflow progress](agent_workflow_inspect(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| session_id | text | None | Trusted session filter |
| principal_ref | text | None | Trusted principal filter |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Workflow summaries, newest metadata included separately |

Use `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Only native workflow checkpoints are listed; unrelated resume records and
corrupt files are omitted. Pass host-attested filters to keep one session or
principal from seeing another's run identifiers.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[List workflow checkpoints](agent_workflow_list(session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
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
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Committed next step or explicit failure |

Use `id`, `step_id`, `response`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

A completed step cannot be committed twice. Replaying the same value returns
the already-committed result; a different value for the same step conflicts.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Validate and commit one step](agent_workflow_commit(id=`id`, step_id=`step_id`, response=`response`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Missing ABI plugin](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## cancel

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable run ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Cancelled checkpoint status |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Cancelling prevents future commits. It does not claim that children already
started by the supervisor were rolled back.

We retain **p = [Locate the Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Commit cancellation](agent_workflow_cancel(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
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
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | New checkpoint state, or waiting_input for the UI |

Use `id`, `source_revision`, `registry`=None, `model`=None, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Advance at most one step. Model text is a candidate, never direct
execution. Waiting for a user returns immediately without holding a thread.
A caller repeats tick after verified completion; UI replies use commit.

`tick` is the only wrapper that may execute a document child, and it only does
so after the checkpoint reports a read-only `ready_tool` step. When the action
is `waiting_input`, return the state unchanged so the host can display the
question and commit the attested answer on a later call. A blocked, uncertain
or cancelled checkpoint is returned as-is; recovery must use `retry_plan` or an
operator decision instead of silently ticking again. The returned state always
describes the committed boundary, not an optimistic guess about a running
child.

We retain **state = [Load the next action](next(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))**.
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
    Return *[Commit the model choice](commit(id=`id`, step_id=`step_id`, response=`chosen`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
  2. *
    Return *answer*.
2. `status` == "ready_tool"
  We retain **step = [Read tool step](json.get(value=`state`, key="step"))**.
  We retain **kind = [Read operation kind](json.get(value=`step`, key="kind"))**.
  We retain **step_id = [Read step identifier](json.get(value=`step`, key="id"))**.
  1. `kind` == "parallel"
    We retain **tasks = [Read child tasks](json.get(value=`step`, key="tasks"))**.
    We retain **report = [Run bounded parallel documents](flow.parallel(tasks=`tasks`))**.
    Return *[Commit all child outcomes](commit(id=`id`, step_id=`step_id`, response=`report`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
  2. `kind` == "skill"
    We retain **name = [Read registered skill name](json.get(value=`step`, key="skill"))**.
    We retain **out = [Run registered SKILL only](flow.enter(skill=`name`, registry=`registry`))**.
    Return *[Commit verified child result](commit(id=`id`, step_id=`step_id`, response=`out`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
  3. *
    Return *state*.
3. `status` == "ready_return"
  We retain **step = [Read return step](json.get(value=`state`, key="step"))**.
  We retain **step_id = [Read step identifier](json.get(value=`step`, key="id"))**.
  We retain **value = [Read final value](json.get(value=`step`, key="value"))**.
  Return *[Commit the completed workflow](commit(id=`id`, step_id=`step_id`, response=`value`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
4. *
  Return *state*.


## retry_plan

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Blocked workflow ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Failed-only read-task list or manual_review |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Only a parallel step explicitly declared read-only and with no uncertain
effects may return a retry plan. Completed siblings are not included.

We retain **p = [Locate Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Select failed read-only children](agent_workflow_retry_plan(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Native plugin unavailable](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## retry_commit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Blocked workflow ID |
| results | list | required | New outcomes for failed children only |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Merged persistent report, or still blocked |

Use `id`, `results`, `source_revision`, `session_id`=None, `principal_ref`=None, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Re-use successful child results from the checkpoint. New results fill failed
slots only, in original order; a newly uncertain outcome blocks further work.

We retain **p = [Locate Agent ABI plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load Agent ABI plugin](plugin.load(path=`p`))**.
  Return *[Merge verified read-only retries](agent_workflow_retry_commit(id=`id`, results=`results`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *[Native plugin unavailable](json.parse(text="{\"ok\":false,\"status\":\"unavailable\"}"))*.

## retry_failed

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Blocked workflow ID |
| source_revision | text | required | Pinned source revision |
| session_id | text | None | Trusted session |
| principal_ref | text | None | Trusted principal |
| limit | number | 2 | Max concurrent failed children |
| timeout_ms | number | 30000 | Child timeout |
| resume_dir | text | ".marqdo/agent-resume" | Host-relative checkpoint directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Recovered workflow, partial results or manual review |

Use `id`, `source_revision`, `session_id`=None, `principal_ref`=None, `limit`=2, `timeout_ms`=30000, `resume_dir`=".marqdo/agent-resume" as the inputs for this operation.

Run only the failed read-only tasks returned by the native checkpoint checker.
Unknown external writes never reach this path.

We retain **plan = [Inspect safe retry manifest](retry_plan(id=`id`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))**.
We retain **can_retry = [Check permitted recovery](json.get(value=`plan`, key="ok"))**.
1. `can_retry`
  We retain **tasks = [Read failed-only children](json.get(value=`plan`, key="tasks"))**.
  We retain **outcomes = [Execute failed read-only children only](subtask.run_many(tasks=`tasks`, limit=`limit`, timeout_ms=`timeout_ms`))**.
  Return *[Merge with preserved completed children](retry_commit(id=`id`, results=`outcomes`, source_revision=`source_revision`, session_id=`session_id`, principal_ref=`principal_ref`, resume_dir=`resume_dir`))*.
2. *
  Return *plan*.
