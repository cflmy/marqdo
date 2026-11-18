---
markup: occlusion
title: two same-lang sessions do not share bindings
import:
  foreign: lib/foreign.mq.md
---

# main

**a = > foreign.session_open lang="node" timeout=30**
**b = > foreign.session_open lang="node" timeout=30**
**> foreign.session_eval id=`a` source="var shared = 41"**
**ra = > foreign.session_eval id=`a` source="shared"**
**rb = > foreign.session_eval id=`b` source="shared"**
**> print text=[result](access(`ra`, "result"))**
**> print text=[ok](access(`rb`, "ok"))**
**> foreign.session_close id=`a`**
**> foreign.session_close id=`b`**
