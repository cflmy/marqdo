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
