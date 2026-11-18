---
markup: occlusion
title: foreign session_eval result outside V_safe is a fault
import:
  foreign: lib/foreign.mq.md
---

# main

**py = > foreign.session_open lang="python" timeout=30**
**> foreign.session_eval id=`py` source="2**60 + 1"**
**> foreign.session_close id=`py`**
