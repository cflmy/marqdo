---
title: time format/parse
import time:lib/time.mq.md
---

# main

*`s` = > time.format unix=0 pattern=%Y-%m-%d *

> print text=`s`

*`u` = > time.parse text=1970-01-01 00:00:00 pattern=%Y-%m-%d %H:%M:%S *

> print text=`u`

> time.sleep_ms ms=0
