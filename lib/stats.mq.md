---
markup: occlusion
title: lib/stats
description: Thin mean / median / sample stdev (Mid2 M10). Not a dataframe library.
---

Thin numeric summaries. Not a dataframe library.

Compute small descriptive statistics.

Behavior and limits: mean/median require nonempty numeric lists. This is not a dataframe or numerical array runtime; ext/linalg owns matrix operations.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/stats.rs](../src/host/stats.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## mean

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Arithmetic mean of list |

Use `list` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/stats.mq.md mean 'list=[1,2,3]' --effects none --json
```

Arithmetic mean of list.

Caller: [stats.mean] list=`xs`

Return *[Arithmetic mean of list](> host_stats_mean list=`list`)*.

## median

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Median of list |

Use `list` as the inputs for this operation.

Effects: pure.

Median of list.

Return *[Median of list](> host_stats_median list=`list`)*.

## stdev

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Sample standard deviation; needs at least two values |

Use `list` as the inputs for this operation.

Effects: pure.

Sample standard deviation; needs at least two values.

Return *[Sample standard deviation; needs at least two values](> host_stats_stdev list=`list`)*.
