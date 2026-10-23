---
markup: occlusion
title: foreign persistent session
import:
  foreign: lib/foreign.mq.md
---

# main

**会话 = > foreign.session_open lang="node"**
**设值 = > foreign.session_eval id=`会话` source="let x = 41"**
**求值 = > foreign.session_eval id=`会话` source="x + 1"**

> print text=[result](access(`求值`, "result"))

**取x = > foreign.session_get id=`会话` name="x"**
> print text=`取x`

**> foreign.session_set id=`会话` name="y" value=5**
**取值 = > foreign.session_get id=`会话` name="y"**

> print text=`取值`

**会话表 = > foreign.session_list**
**数量 = > len `会话表`**
> print text=`数量`

**关闭 = > foreign.session_close id=`会话`**
> print text=`关闭`
