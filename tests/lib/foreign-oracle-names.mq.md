---
markup: occlusion
title: explicit names response is a decode observation
import:
  foreign: lib/foreign.mq.md
---

# main

**s = > foreign.session_open lang="python"**
**> foreign.session_set id=`s` name="zzz" value=7**
**> foreign.session_names id=`s`**
**j = > foreign.oracle_journal**
**> print text=[channel](access(access(`j`, 1), "channel"))**
**> print text=[name](access(access(access(`j`, 1), "value"), 2))**
**> foreign.session_close id=`s`**
