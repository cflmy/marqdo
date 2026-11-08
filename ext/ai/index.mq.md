---
markup: occlusion
title: AI workflows — model adapters, agents, workbooks and trials
import:
  model_api: ext/ai/llm.mq.md
---

Start here to choose an AI task. [Model calls](llm.mq.md) own prompts, model configuration,
stream collection and usage; [provider adapters](llm/index.mq.md) describe OpenAI-compatible
and Ollama requests. [Agents](agent.mq.md) own call-site context, the explicit tool registry,
policy routing, observations, workbook planning and continuation. [CoW trials](sandbox.mq.md)
own isolated file changes and promotion. [Step routing](policies/step-route.mq.md) is readable
policy data, not a second agent language. Chinese users can start at [索引](索引.mq.md).

Begin with this offline stream example; it does not request credentials, install a plugin or
contact a model. Commands run from the repository root:

```sh
marqdo call ext/ai/index.mq.md offline_example --json
```

For a live model call, configure its provider and credentials before invoking the existing llm
resource. A configured language or adapter does not prove a remote endpoint is available.
Agent operations require the native agent plugin, an explicit model and the permitted tools;
use cargo build -p marqdo_plugin_agent in this checkout. Never let a generated CALL line select
an arbitrary internal function. File, process and network permissions remain bounded by the host.

Develop one responsibility at a time: provider request shaping belongs under llm/, common model
state in llm.mq.md, planner orchestration in agent.mq.md and trial behavior in sandbox.mq.md.
Keep the agent's call-site and source budgets intact when changing its internal helpers; moving
an operation can affect context and replay. Review effect preparation, transitions and saved
results before adding retries, and do not repeat writes merely to reconstruct a checkpoint.

Use deterministic stream fixtures and local provider substitutes before live tests. Verify text,
usage, failed tools, lost responses, cancellation, context limits and durable continuation;
model output is data until an authorized tool target and arguments have been resolved.

## collect_events

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| events | list | required | Ordered semantic or supported provider stream-event records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Final text assembled by the public stream collector |

The supplied `events` are an already-observed stream, not instructions to request a new model
response or execute a tool. The shared collector understands delta/text, done.result and its
existing provider-frame conventions. We return *[Collect the existing stream into final text](model_api.collect(events=`events`))*
so offline processing and live transport use the same public assembly behavior.

## offline_example

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Hello world, assembled from actual semantic events without a model request |

The event table is a deterministic stream fixture. A delta introduces partial text, then the
final done result supplies the completed answer. Passing these through the public collector
verifies useful behavior; this tool does not manufacture a passed flag or a fake model answer.

`events` =

| @ | type | text | result |
|---|---|---|---|
| 1 | delta | "Hello " | None |
| 2 | done | None | "Hello world" |

We return *[Assemble the final text from the displayed stream](collect_events(events=`events`))*.
A real transport failure still needs its domain error handling; this fixture only demonstrates
stream assembly, and it deliberately performs no network operation.
