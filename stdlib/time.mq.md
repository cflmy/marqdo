---
title: lib/time — clock
description: Unix time, format, parse, sleep
import time:lib/time.mq.md
---

# main

Unix time helpers from lib/time. Patterns use strftime.

**`s` = [time.format] unix=0 pattern="%Y-%m-%d"**
**打印 内容=`s`**

**`u` = [time.parse] text="1970-01-01 00:00:00" pattern="%Y-%m-%d %H:%M:%S"**
**打印 内容=`u`**

**[time.sleep_ms] ms=0**
