---
markup: occlusion
title: nested ### sub-tool call
---

# main

*None*

## outer

Parent tool. Agents may call this path or the finer nested helper.

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | seed value |

This paragraph is narration so the unit stays readable. The nested
`### helper` is the sub-tool that does the arithmetic. Call path
`outer.helper` selects the child without running the parent body.

### helper

| 参数 | 类型 | 说明 |
|------|------|------|
| n | number | value to double |

Sub-tool under `outer`. Doubles `n` without side effects so default
`marqdo call` may run it. Prefer this when only the doubling step is needed.

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | two times n |

*[result](n * 2)*

| 返回 | 类型 | 说明 |
|------|------|------|
|  | number | parent returns n unchanged |

*n*
