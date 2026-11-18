---
markup: occlusion
title: closed handle cannot be reused across a later step
import:
  foreign: lib/foreign.mq.md
---

# main

**会话 = > foreign.session_open lang="node" timeout=30**
**> foreign.session_eval id=`会话` source="var x = 1"**
**> foreign.session_close id=`会话`**
**> foreign.session_eval id=`会话` source="x"**
