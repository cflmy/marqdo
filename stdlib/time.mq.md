---
title: lib/time — clock
description: Unix time, format, parse, sleep
> lib/time.mq.md
---

# main

Import lib/time.mq.md. Functions: now_unix, now_ms, format(unix, pattern), parse(text, pattern), sleep_ms(ms). Patterns use strftime.

*`s` = > time.format unix=0 pattern=%Y-%m-%d *

> print text=`s`

*`u` = > time.parse text=1970-01-01 00:00:00 pattern=%Y-%m-%d %H:%M:%S *

> print text=`u`

> time.sleep_ms ms=0
