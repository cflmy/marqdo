---
title: time format/parse
> lib/time.mq.md
---

# main

*`s` = > format unix=0 pattern=%Y-%m-%d *

> print text=`s`

*`u` = > parse text=1970-01-01 00:00:00 pattern=%Y-%m-%d %H:%M:%S *

> print text=`u`

> sleep_ms ms=0
