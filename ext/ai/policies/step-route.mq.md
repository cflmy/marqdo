---
markup: occlusion
title: step-route policy (O3)
description: >
  Table-driven map from step class → Adaptive Routing backend.
  Readable authority for agent.step_route; do not invent a second dialect.
---

# Step class → backend

Authors and `agent.step_route` read this table. Backends match Adaptive Routing:
`marqdo` | `small-llm` | `llm` | `auto`.

**routes = **

| kind | backend | note |
|------|---------|------|
| lookup | marqdo | OKF / kb skill hit — no parent LLM |
| patch | small-llm | Local patch / soft match |
| sandbox_run | marqdo | CoW trial of `.mq.md` |
| net | llm | Needs network / toolful turn |
| default | auto | Cascade policy → kb → small-llm |

# main

*routes*
