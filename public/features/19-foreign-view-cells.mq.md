---
title: Foreign kernels v2 — Python analysis in view
description: Kernel state, Marqdo-to-kernel values and view cells — without a stdin-only demo
import foreign:lib/foreign.mq.md
---

# main

This page shows the foreign runtime as a language feature: a persistent kernel, explicit state from Marqdo values, and view cells that share kernel state. It never asks the reader to type JSON into stdin.

---

Probe the interpreter first. The result is ordinary Marqdo data.

**python = > foreign.probe lang="python"**

> print text=[ok](`python`)
> print text=[version](`python`)

---

Marqdo values into a kernel.

The `订单` table is a normal Marqdo value. `foreign.session_set` copies it into the Python kernel as JSON; no shell and no stdin.

`订单` =

| @ | 区域 | 金额 |
|---|------|------|
| 1 | 华东 | 120 |
| 2 | 华东 | 300 |
| 3 | 华南 | 90 |
| 4 | 华南 | 140 |
| 5 | 华北 | 220 |
| 6 | 西部 | 80 |

**内核会话 = > foreign.session_open lang="python"**

**> foreign.session_set id=`内核会话` name="orders" value=`订单`**

**total = > foreign.session_eval id=`内核会话` source="round(sum(float(row['金额']) for row in orders), 2)"**

> print text=[result](`total`)

The low-level kernel session is managed entirely by `foreign`; the table crossed the boundary as an explicit JSON copy.

---

Kernel state persists.

Two evaluations share the same kernel. The second sees the first variable.

**> foreign.session_eval id=`内核会话` source="values = [3, 4, 5]"**

**square_sum = > foreign.session_eval id=`内核会话` source="sum(value * value for value in values)"**

> print text=[result](`square_sum`)

**> foreign.session_close id=`内核会话`**

This is kernel state, not a one-shot process. It is created, queried and closed through `foreign`.

---

View cells share state.

`foreign.session` declares a view session from ordinary tables. The table names are free; this document uses `分析内核` and `分析单元`.

`分析内核` =

| @ | 名称 | 语言 |
|---|------|------|
| 1 | 分析 | python |

`分析单元` =

| @ | 单元 | 内核 |
|---|------|------|
| 1 | 准备 | 分析 |
| 2 | 计算 | 分析 |

`会话` = > foreign.session kernels=`分析内核` cells=`分析单元`

`准备` =
```python
orders = [
    {"区域": "华东", "金额": 120},
    {"区域": "华东", "金额": 300},
    {"区域": "华南", "金额": 90},
    {"区域": "华南", "金额": 140},
    {"区域": "华北", "金额": 220},
    {"区域": "西部", "金额": 80},
]
```

`计算` =
```python
import json
import statistics

by_region = {}
for row in orders:
    by_region.setdefault(row["区域"], []).append(float(row["金额"]))

summary = []
for region, values in sorted(by_region.items()):
    mean = statistics.mean(values)
    sd = statistics.pstdev(values) or 1.0
    summary.append({
        "region": region,
        "orders": len(values),
        "total": round(sum(values), 2),
        "mean": round(mean, 2),
        "z_span": round((max(values) - mean) / sd, 3),
    })

print(json.dumps({
    "regions": summary,
    "total": round(sum(float(row["金额"]) for row in orders), 2),
    "count": len(orders),
}, ensure_ascii=False))
```

`查看` consumes the `会话` handle. Run All runs `准备` then `计算` in the same Python kernel; the static export runs the same cells at build time and embeds their real outputs.

The important language point:

Tables are values passed to `foreign.session`. Kernel state lives in `foreign`. View only renders the handle and its cells. `run` still executes the same document as a program.

There is no separate notebook CLI, no stdin requirement in this example, and no hidden language switch.
