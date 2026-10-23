---
markup: occlusion
title: foreign session timeout
import:
  foreign: lib/foreign.mq.md
---

# main

**会话 = > foreign.session_open lang="node"**
**慢 = > foreign.session_eval id=`会话` source="(async () => { await new Promise(r => setTimeout(r, 5000)); return 1; })()" timeout=0.3**

> print text=[ok](access(`慢`, "ok"))
> print text=[timed_out](access(`慢`, "timed_out"))
> print text=[alive](access(`慢`, "alive"))
