---
markup: occlusion
title: Foreign kernels v2 — Python analysis in view
description: Kernel state, Marqdo-to-kernel values and view cells — without a stdin-only demo
import:
  foreign: lib/foreign.mq.md
---

# main

This page shows the foreign runtime as a language feature: a persistent kernel, explicit state from Marqdo values, and view cells that share kernel state. It never asks the reader to type JSON into stdin.

---

Probe the interpreter first. The result is ordinary Marqdo data.

**python = > foreign.probe lang="python"**

> print text=[ok](access(`python`, "ok"))
> print text=[version](access(`python`, "version"))

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

`total_code` =
```python
round(sum(float(row["金额"]) for row in orders), 2)
```

**total = > foreign.session_eval id=`内核会话` code=`total_code`**

> print text=[result](access(`total`, "result"))

The low-level kernel session is managed entirely by `foreign`; the table crossed the boundary as an explicit JSON copy.

---

Kernel state persists.

Two evaluations share the same kernel. The second sees the first variable.

`set_values_code` =
```python
values = [3, 4, 5]
```

**> foreign.session_eval id=`内核会话` code=`set_values_code`**

`square_sum_code` =
```python
sum(value * value for value in values)
```

**square_sum = > foreign.session_eval id=`内核会话` code=`square_sum_code`**

> print text=[result](access(`square_sum`, "result"))

**> foreign.session_close id=`内核会话`**

This is kernel state, not a one-shot process. It is created, queried and closed through `foreign`.

---

View cells share state.

`foreign.session` declares a view session from ordinary tables. The table names are free; this document uses `分析内核` and `分析单元`.

`分析内核` =

| @ | 名称 | 语言 |
|---|------|------|
| 1 | 分析 | python |
| 2 | 复核 | python |

`分析单元` =

| @ | 单元 | 内核 |
|---|------|------|
| 1 | 计算 | 分析 |
| 2 | 复核 | 复核 |

`初始值` =

| 内核 | 名称 | 值 |
|------|------|----|
| 分析 | orders | `订单` |
| 复核 | orders | `订单` |

`会话` = > foreign.session kernels=`分析内核` cells=`分析单元` values=`初始值`

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

`复核` =
```python
import json

print(json.dumps({
    "kernel": "复核",
    "count": len(orders),
    "total": round(sum(float(row["金额"]) for row in orders), 2),
}, ensure_ascii=False))
```

`查看` consumes the `会话` handle. Run All runs `计算` in the `分析` kernel and `复核` in the `复核` kernel; the static export runs the same cells and embeds their real outputs.

The important language point:

The `初始值` table passes the same Marqdo `订单` value into both kernels. Kernel state lives in `foreign`. View only renders the handle and its cells. `run` still executes the same document as a program.

There is no separate notebook CLI, no stdin requirement in this example, and no hidden language switch.
