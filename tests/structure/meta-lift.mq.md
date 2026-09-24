---
title: meta-lift
model: ${env.MARQDO_LIFT_MODEL ?? "lifted-model"}
api_key: ${secret.MARQDO_LIFT_SECRET ?? "sk-lift"}
greeting: "hi"
---

# main

Metadata keys are ordinary variables after Phase 2 lift.

> print text=`model`
> print text=`greeting`
> print text=`api_key`
