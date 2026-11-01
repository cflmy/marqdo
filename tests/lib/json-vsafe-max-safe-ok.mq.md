---
markup: occlusion
title: json V_safe MAX_SAFE roundtrip
import:
  json: lib/json.mq.md
---

# main

**obj = > json.parse text={"n":9007199254740991}**
**n = > json.get value=`obj` key="n"**
**out = > json.stringify value=`obj`**

> print text=`n`
> print text=`out`
