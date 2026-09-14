---
title: lib/csv smoke
import csv:lib/csv.mq.md
---

# main

**raw = "name,note\nalice,\"hi, there\"\nbob,ok\n"**

**rows = > csv.parse text=`raw`**
**n = > len `rows`**
> print text=`n`

**r0 = > at value=`rows` index=0**
> print text=[name](`r0`)
> print text=[note](`r0`)

**out = > csv.stringify rows=`rows`**
**rows2 = > csv.parse text=`out`**
**n2 = > len `rows2`**
> print text=`n2`
