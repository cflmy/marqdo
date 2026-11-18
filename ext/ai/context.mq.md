---
markup: occlusion
title: AI context — short, explicit source scopes
description: Pack user input and caller-provided file/class/function excerpts.
import:
  agent: ext/ai/agent.mq.md
  plugin: lib/plugin.mq.md
  json: lib/json.mq.md
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


## scope

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| scope | text | "function" | file / class / function / tool |
| class | text | None | Class heading for disambiguation |
| function | text | None | Function/tool heading |
| source | text | None | Source override; otherwise current entry module |
| max_chars | number | 1600 | Maximum Unicode characters |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Exact source excerpt, line range, or explicit ambiguity |

Use `scope`="function", `class`=None, `function`=None, `source`=None, `max_chars`=1600 as the inputs for this operation.

The native selector respects Markdown headings and ignores fenced-code heading
lookalikes. Repeated function names require a class; missing/ambiguous scopes
return an error value, never an unrelated or whole-file fallback.

We retain **p = [Locate native agent plugin](plugin.native_path(name="agent"))**.
1. `p`
  We **[Load native agent plugin](plugin.load(path=`p`))**.
2. *
  Return *[No native parser available](json.parse(text="{\"ok\":false,\"status\":\"unavailable\",\"text\":null}"))*.

Return *[Project the requested document scope](agent_context_scope(source=`source`, scope=`scope`, class=`class`, function=`function`, max_chars=`max_chars`))*.
