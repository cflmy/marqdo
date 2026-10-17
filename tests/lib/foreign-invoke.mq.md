---
title: foreign invoke record
import foreign:lib/foreign.mq.md
---

# main

`失败` =
```node
process.stderr.write("boom");
process.exit(3);
```

**记录 = > foreign.invoke code=`失败`**

> print text=[ok](`记录`)
> print text=[code](`记录`)
> print text=[stderr](`记录`)
> print text=[truncated](`记录`)
