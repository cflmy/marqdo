---
markup: occlusion
title: foreign invoke record
import:
  foreign: lib/foreign.mq.md
---

# main

`失败` =
```node
process.stderr.write("boom");
process.exit(3);
```

**记录 = > foreign.invoke code=`失败`**

> print text=[ok](access(`记录`, "ok"))
> print text=[code](access(`记录`, "code"))
> print text=[stderr](access(`记录`, "stderr"))
> print text=[truncated](access(`记录`, "truncated"))
