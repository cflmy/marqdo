---
title: O3 step_route table backends
description: lookup → marqdo; net → llm; default → auto.
import agent:ext/ai/agent.mq.md
---

# main

**a = agent.step_route kind="lookup"**
**ba = [backend](a)**
> print text=`ba`

**b = agent.step_route kind="net"**
**bb = [backend](b)**
> print text=`bb`

**c = agent.step_route kind="default"**
**bc = [backend](c)**
> print text=`bc`
