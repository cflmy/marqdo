---
title: json roundtrip
> lib/fs.mq.md
> lib/json.mq.md
---

# main

*`raw` = > read_text path=sample.json *

*`obj` = > parse text=`raw` *

*`ty` = > type `obj` *

> print text=`ty`

*`a` = > get value=`obj` key=a *

> print text=`a`

*`out` = > stringify value=`obj` *

> print text=`out`
