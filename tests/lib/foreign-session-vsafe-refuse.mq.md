---
markup: occlusion
title: foreign session_set V_safe refuse
import:
  foreign: lib/foreign.mq.md
---

# main

**node = > foreign.session_open lang="node" timeout=30**
**n = 1152921504606846977**
**> foreign.session_set id=`node` name="__n" value=`n`**
**> foreign.session_close id=`node`**
