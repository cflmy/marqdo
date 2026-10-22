---
markup: occlusion
title: foreign python session
import foreign:lib/foreign.mq.md
---

# main

**会话 = > foreign.session_open lang="python"**
**设值 = > foreign.session_eval id=`会话` source="x = 41"**
**求值 = > foreign.session_eval id=`会话` source="x + 1"**

> print text=[result](access(`求值`, "result"))

**> foreign.session_set id=`会话` name="y" value=5**
**取值 = > foreign.session_get id=`会话` name="y"**

> print text=`取值`
**> foreign.session_close id=`会话`**
