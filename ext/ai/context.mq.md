---
markup: occlusion
title: AI context — short, explicit source scopes
description: Pack user input and caller-provided file/class/function excerpts.
import:
  agent: ext/ai/agent.mq.md
---

Pass only the documentation that the next model decision needs. The supplied
file/class/function snippets are data; they do not execute as Marqdo source.
A future compiler-backed selector can supply these arguments from AST spans.
This helper does not yet infer class/function scopes automatically.

## pack

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| user | text | required | Latest user input |
| file | text | "" | File-level description excerpt |
| class | text | "" | Current class excerpt |
| function | text | "" | Current function excerpt |
| budget_tokens | number | 2048 | Small-model context budget |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Bounded ContextPackV2 and compaction evidence |

Use `user`, `file`="", `class`="", `function`="", `budget_tokens`=2048 as the inputs for this operation.

Keep the latest request and current function first. Low-priority file context
may be dropped under pressure; any important constraint must be in a retained
field, not in a discardable tool transcript.

`items` =

| @ | origin | trust | priority | text | retain |
|---|---|---|---|---|---|
| 1 | user | user | 0 | `user` | True |
| 2 | function | unknown | 1 | `function` | True |
| 3 | class | unknown | 2 | `class` | False |
| 4 | file | unknown | 3 | `file` | False |

Return *[Pack the supplied scopes and compact redundant context](agent.context_pack_compact(items=`items`, budget_tokens=`budget_tokens`, goal=`user`))*.
