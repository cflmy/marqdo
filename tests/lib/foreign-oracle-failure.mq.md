---
markup: occlusion
title: failed foreign responses remain observable values
import:
  foreign: lib/foreign.mq.md
---

# main

**s = > foreign.session_open lang="python"**
**> foreign.session_eval id=`s` source="raise ValueError('outside V_safe')"**
**j = > foreign.oracle_journal**
**> print text=[channel](access(access(`j`, 1), "channel"))**
**> print text=[ok](access(access(access(`j`, 1), "value"), "ok"))**
**> foreign.session_eval id=`s` source="import os; os._exit(7)"**
**later = > foreign.oracle_journal**
**> print text=[alive](access(access(access(`later`, 2), "value"), "alive"))**
