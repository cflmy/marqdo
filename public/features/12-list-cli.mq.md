---
title: Mid stdlib — lists and cli
description: table sort/unique/zip + cli.parse (Mid M5)
import table:lib/table.mq.md
import csv:lib/csv.mq.md
import cli:lib/cli.mq.md
---

# Sort CSV rows and parse flags

Zero plugins: parse CSV, sort by key, parse `--input`.

# main

*raw = "name,score\nbob,2\nalice,10\n"*
*rows = > csv.parse text=`raw`*
*rows = > table.sort_by list=`rows` key="name"*
*r0 = > at value=`rows` index=0*
> print text=`r0`[^name]

*args = > split value="--input,scores.csv,--pretty" sep=","*
*flags = > cli.parse args=`args`*
> print text=`flags`[^input]
1. `flags`[^pretty]
  > print text=mid-m5-ok
2. *
  > print text=mid-m5-fail
