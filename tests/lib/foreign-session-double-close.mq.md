---
markup: occlusion
title: foreign session double-close is a fault
import:
  foreign: lib/foreign.mq.md
---

# main

**会话 = > foreign.session_open lang="node" timeout=30**
**> foreign.session_close id=`会话`**
**> foreign.session_close id=`会话`**
