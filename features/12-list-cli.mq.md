---
markup: occlusion
title: Mid stdlib — lists and cli
description: table sort/unique/zip + cli.parse (Mid M5)
import:
  table: lib/table.mq.md
  csv: lib/csv.mq.md
  cli: lib/cli.mq.md
---

# Sort CSV rows and parse flags


Zero plugins: parse CSV, sort by key, parse `--input`.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **raw = "name,score\nbob,2\nalice,10\n"** for the next step.
We first **rows = [Compute rows](csv.parse(text=`raw`))** for the next step.
We first **rows = [Compute rows](table.sort_by(list=`rows`, key="name"))** for the next step.
We first **r0 = [Compute r0](at(value=`rows`, index=0))** for the next step.
We now **[Display this step's result](> print text=[name](access(`r0`, "name")))**.

We first **args = [Compute args](split(value="--input,scores.csv,--pretty", sep=","))** for the next step.
We first **flags = [Compute flags](cli.parse(args=`args`))** for the next step.
We now **[Display this step's result](> print text=[input](access(`flags`, "input")))**.
1. [pretty](access(`flags`, "pretty"))
  We now **[Display this step's result](> print text="mid-m5-ok")**.
2. *
  We now **[Display this step's result](> print text="mid-m5-fail")**.
