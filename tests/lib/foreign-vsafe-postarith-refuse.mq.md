---
markup: occlusion
title: foreign V_safe refuse after Python arithmetic
import:
  foreign: lib/foreign.mq.md
  json: lib/json.mq.md
---

# main

**py = > foreign.session_open lang="python" timeout=30**

`emit` =
```python
import json
n = 2**53 - 1
print(json.dumps({"n": 2 * n}))
```

**raw = > foreign.session_eval id=`py` code=`emit`**
**stdout = > json.get value=`raw` key="stdout"**
**payload = > json.parse text=`stdout`**
**> print text=`payload`**
**> foreign.session_close id=`py`**
