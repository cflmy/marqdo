---
title: Foreign kernels v2 — Python analysis in view
description: A complete Python data walkthrough with probe, JSON calls, parallel fan-out, persistent state and view cells
import foreign:lib/foreign.mq.md
---

# main

This page is both a readable document and a runnable program. It uses the same file to show the foreign-kernel features added in v2: language probing, structured records, JSON in/out, parallel fan-out, a persistent Python kernel, and an explicit state bridge. The same source can also be opened in view as interactive cells.

The data is intentionally small and deterministic, so every number below can be reproduced by running the page.

---

The input.

`orders` =

| @ | region | amount |
|---|--------|--------|
| 1 | 华东 | 120 |
| 2 | 华东 | 300 |
| 3 | 华南 | 90 |
| 4 | 华南 | 140 |
| 5 | 华北 | 220 |
| 6 | 西部 | 80 |

---

Probe Python before using it.

**python = > foreign.probe lang="python"**

> print text=[ok](`python`)
> print text=[version](`python`)

The probe record also carries `cmd`, `path` and `error`, so a site can explain a missing interpreter instead of producing a blank cell.

---

A JSON call. Python reads the table as JSON, groups it, and writes one JSON result.

`analyze` =
```python
import json
import statistics
import sys

rows = json.load(sys.stdin)
by_region = {}
for row in rows:
    by_region.setdefault(row["region"], []).append(float(row["amount"]))

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
    "total": round(sum(float(row["amount"]) for row in rows), 2),
    "count": len(rows),
}, ensure_ascii=False))
```

**result = > foreign.call code=`analyze` input=`orders`**

> print text=[count](`result`)
> print text=[total](`result`)
> print text=[region]([1]([regions](`result`)))
> print text=[mean]([1]([regions](`result`)))

A call is JSON-in / JSON-out: the table is encoded on stdin, and the final stdout value is parsed into Marqdo values.

---

The same work as a structured record. `invoke` keeps stdout, stderr, exit code and timing even when the program fails.

**record = > foreign.invoke code=`analyze` input=`orders` output="json"**

> print text=[ok](`record`)
> print text=[code](`record`)
> print text=[ms](`record`)

Here `result` is already a parsed value, while `stdout` remains available for logging.

---

Persistent Python state. The first evaluation imports a module and creates a list; the second evaluation sees both.

**session = > foreign.session_open lang="python"**

**first = > foreign.session_eval id=`session` source="import math\nvalues = [3, 4, 5]"**

**second = > foreign.session_eval id=`session` source="round(math.sqrt(sum(value * value for value in values)), 3)"**

> print text=[result](`second`)

The kernel is strict and serial: evaluations do not interleave, and a timeout kills the process group.

---

The explicit state bridge. Values cross as JSON copies, never as shared object references.

**> foreign.session_set id=`session` name="threshold" value=2.5**

**threshold = > foreign.session_get id=`session` name="threshold"**

> print text=`threshold`

**> foreign.session_close id=`session`**

No implicit mirroring: if a variable is not pushed or pulled, the other side does not see it.

---

Parallel fan-out. Every input gets its own process; failures are isolated and order is preserved.

`increment` =
```python
import json
import sys

value = json.load(sys.stdin)
print(json.dumps(value + 1))
```

`numbers` =

| value |
|-------|
| 1 |
| 2 |
| 3 |
| 4 |

**bumped = > foreign.call_many code=`increment` inputs=`numbers` concurrency=4**

> print text=`bumped`

---

Cells in view. Named kernels and a cell order are ordinary bound tables, not fence metadata or YAML config.

```text
`内核表` =

| 名称 | 语言 | 命令 | 说明 |
|------|------|------|------|
| 分析 | python | | shared analysis |
| 复核 | python | | isolated review |

`会话表` =

| 单元 | 内核 | 说明 |
|------|------|------|
| 准备数据 | 分析 | generate orders |
| 计算指标 | 分析 | regions and totals |
| 独立复核 | 复核 | read the JSON report |
```

`marqdo view` renders these as cells with Run All, Step, per-cell kernel switching, a journal, dirty reload and a state panel. There is no separate interactive command; `marqdo run` still executes the same file as one program.

---

Why this stays readable.

| Layer | What the reader sees |
|---|---|
| Markdown | headings, prose, tables and fenced code |
| Program | assignments, calls and return values |
| Knowledge | frontmatter, links, task pages and generated catalog entries |

The foreign layer does not add syntax to the language. A fence is still a fence; a table is still a table.

For the lower-level API and safety defaults, read [`../../lib/foreign.mq.md`](../../lib/foreign.mq.md) and [`../../doc/design/foreign-kernel.md`](../../doc/design/foreign-kernel.md).
