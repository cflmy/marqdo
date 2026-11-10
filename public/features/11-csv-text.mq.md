---
markup: occlusion
title: Mid stdlib — csv and text
description: lib/csv + thickened lib/text (Mid M4)
import:
  csv: lib/csv.mq.md
  text: lib/text.mq.md
---

# CSV cleaning with text helpers


Parse a tiny CSV, normalize a field, print — no plugins.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **raw = "name,city\nAda,london\nBob,PARIS\n"** for the next step.
We first **rows = [Compute rows](csv.parse(text=`raw`))** for the next step.
We first **r0 = [Compute r0](at(value=`rows`, index=0))** for the next step.
We first **city = [Compute city](text.to_upper(text=[city](access(`r0`, "city"))))** for the next step.
We now **[Display this step's result](> print text=[name](access(`r0`, "name")))**.
We now **[Display this step's result](> print text=`city`)**.
We first **n = [Compute n](len(`rows`))** for the next step.
1. `n` == 2
  We now **[Display this step's result](> print text="mid-m4-ok")**.
2. *
  We now **[Display this step's result](> print text="mid-m4-fail")**.
