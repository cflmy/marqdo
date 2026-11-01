---
markup: occlusion
title: foreign Python JSON number V_safe refuse
import:
  foreign: lib/foreign.mq.md
  json: lib/json.mq.md
---

# main

**py = > foreign.session_open lang="python" timeout=30**

`emit` =
```python
import json
print(json.dumps({"n": 2**60 + 1}))
```

**raw = > foreign.session_eval id=`py` code=`emit`**
**stdout = > json.get value=`raw` key="stdout"**
**payload = > json.parse text=`stdout`**
**> print text=`payload`**
**> foreign.session_close id=`py`**
