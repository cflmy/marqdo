---
title: table sort_by + cli
import table:lib/table.mq.md
import csv:lib/csv.mq.md
import cli:lib/cli.mq.md
---

# main

*raw = "name,score\nbob,2\nalice,10\n"*
*rows = > csv.parse text=`raw`*
*rows = > table.sort_by list=`rows` key="name"*
*r0 = > at value=`rows` index=0*
> print text=`r0`[^name]

*args = > split value="--input,data.csv,rest,--verbose" sep=","*
*flags = > cli.parse args=`args`*
> print text=`flags`[^input]
> print text=`flags`[^verbose]
*pos = `flags`[^_]*
*p0 = > at value=`pos` index=0*
> print text=`p0`
