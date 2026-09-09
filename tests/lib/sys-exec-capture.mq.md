---
title: sys.exec capture=True returns stdout map
description: GAP-09 optional capture.
import sys:lib/sys.mq.md
import json:lib/json.mq.md
---

# main

*code = > sys.exec cmd="true"*
1. `code` == 0
  > print text=exec-code-ok
2. *
  > print text=exec-code-fail

*args = > json.parse text=["-c","printf hi-capture"]*
*out = > sys.exec cmd="sh" args=args capture=True*
*c = out[^code]*
*s = out[^stdout]*
1. `c` == 0
  > print text=capture-code-ok
2. *
  > print text=capture-code-fail

1. `s` == "hi-capture"
  > print text=capture-stdout-ok
2. *
  > print text=capture-stdout-fail
