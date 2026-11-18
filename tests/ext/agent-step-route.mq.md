---
markup: occlusion
title: O3 step_route table backends
description: lookup → marqdo; net → llm; default → auto.
import:
  agent: ext/ai/agent.mq.md
---

# main

**a = agent.step_route kind="lookup"**
**ba = [backend](access(a, "backend"))**
> print text=`ba`

**b = agent.step_route kind="net"**
**bb = [backend](access(b, "backend"))**
> print text=`bb`

**c = agent.step_route kind="default"**
**bc = [backend](access(c, "backend"))**
> print text=`bc`
