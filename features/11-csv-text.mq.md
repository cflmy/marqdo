---
title: Mid stdlib — csv and text
description: lib/csv + thickened lib/text (Mid M4)
import csv:lib/csv.mq.md
import text:lib/text.mq.md
---

# CSV cleaning with text helpers

Parse a tiny CSV, normalize a field, print — no plugins.

# main

*raw = "name,city\nAda,london\nBob,PARIS\n"*
*rows = > csv.parse text=`raw`*
*r0 = > at value=`rows` index=0*
*city = > text.to_upper text=`r0`[^city]*
> print text=`r0`[^name]
> print text=`city`
*n = > len `rows`*
1. `n` == 2
  > print text=mid-m4-ok
2. *
  > print text=mid-m4-fail
