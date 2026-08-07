---
title: json roundtrip
> lib/fs.mq.md
> lib/json.mq.md
---

# main

*`raw` = > fs.read_text path=sample.json *

*`obj` = > json.parse text=`raw` *

*`ty` = > type `obj` *

> print text=`ty`

*`a` = > json.get value=`obj` key=a *

> print text=`a`

*`out` = > json.stringify value=`obj` *

> print text=`out`
