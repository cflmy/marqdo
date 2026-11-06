---
markup: occlusion
title: persistent sessions enforce the same code payload refusal
import:
  foreign: lib/foreign.mq.md
---

# main

`payload` =
```python
1
```

**s = > foreign.session_open lang="python"**
**> foreign.session_set id=`s` name="payload" value=`payload`**
