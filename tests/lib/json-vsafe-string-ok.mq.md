---
markup: occlusion
title: json V_safe decimal string ok
import:
  json: lib/json.mq.md
---

# main

**obj = > json.parse text={"s":"1152921504606846977"}**
**s = > json.get value=`obj` key="s"**

> print text=`s`
