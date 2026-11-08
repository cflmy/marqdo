---
markup: occlusion
title: step-route policy (O3)
description: >
  Table-driven map from step class → Adaptive Routing backend.
  Readable authority for agent.step_route; do not invent a second dialect.
---

Step class to backend.

Authors and `agent.step_route` read this table. Backends match Adaptive Routing:
`marqdo` | `small-llm` | `llm` | `auto`.

## routes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Return the same readable route table used by the agent policy reader. |

Return the same readable route table used by the agent policy reader.

Each column is a list with the same row order. Lookup and sandbox execution prefer Marqdo, patch work prefers the small model, and network-oriented work uses the model route. The default row leaves the final choice to the existing router. Returning these values does not load or invoke a backend.

**routes = **

| kind | backend | note |
|------|---------|------|
| lookup | marqdo | OKF / kb skill hit — no parent LLM |
| patch | small-llm | Local patch / soft match |
| sandbox_run | marqdo | "CoW trial of .mq.md" |
| net | llm | Needs network / toolful turn |
| default | auto | Cascade policy → kb → small-llm |

Return *routes*.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The entry returns the policy data without creating a second routing dialect. |

The entry returns the policy data without creating a second routing dialect.

Return *[Routes](routes())*.
