---
markup: occlusion
title: foreign session handle cannot cross JSON bridge
import:
  foreign: lib/foreign.mq.md
  json: lib/json.mq.md
---

# main

**会话 = > foreign.session_open lang="node" timeout=30**
**> json.stringify value=`会话`**
**> foreign.session_close id=`会话`**
