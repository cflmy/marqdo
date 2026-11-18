---
markup: occlusion
title: Native verification for Markdown data refactors
import:
  document: lib/document.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
---

Extract static literal tables into readable Markdown data leaves while callable sources explain their use. Compare actual old function values with the new native parser before applying source changes. These pure tasks do not execute the supplied data as a program or mutate files.

## literal_tables

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| specifications | list | required | Records with a unique key and literal Markdown table text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing ordered key/value records for all selected literal tables |

Use `specifications` to inspect a planned Markdown data split before writing it. Each key identifies the original file and function, while text carries that table's exact source. The standard document parser preserves native literal types and shapes and refuses dynamic expressions. No file, import, process or code fence is executed. Duplicate identities are a failed Result rather than an overwritten observation, and parser diagnostics retain their actual failure. This task is pure; it supplies data evidence, not permission to replace source.

We **rows = [Start the ordered literal evidence](table.empty_list())** and **keys = [Track exact source identities](table.empty_list())**.
- [Read every explicitly selected literal table](for specifications as specification)
  We **key = [Read its source identity](access(`specification`, "key"))** and **source = [Read its exact table source](access(`specification`, "text"))**.
  1. [This identity was already observed](table.contains(list=`keys`, item=`key`))
    Return *[Keep duplicate source identity explicit](result.error(code="data.duplicate", message=`key`, operation="literal_tables"))*.
  We **value = [Use the native literal table parser](document.table(text=`source`))** and **keys = [Retain this identity](table.append(list=`keys`, item=`key`))**.

  `row` =

  | key | value |
  |---|---|
  | `key` | `value` |

  We **rows = [Keep the complete native value](table.append(list=`rows`, item=`row`))**.
Return *[Expose ordered immutable data evidence](result.ok(value=`rows`))*.

## compare

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expected | list | required | Baseline key/value records obtained from actual original functions |
| actual | list | required | Key/value records obtained from the proposed literal data |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with exact count and the source keys whose native values differ |

Use `expected` and `actual` for a data-only refactor review. Identities and cardinality are checked before any source mutation, then values are compared through native equality. A boolean must remain a boolean, and row/column/list structure must remain the same. The failed Result keeps mismatched identities in value, allowing --require-ok to stop a migration without losing evidence. This task neither writes files nor declares a later behavioral change equivalent; deliberate content edits need their own review.

We **mismatched = [Start actual data differences](table.empty_list())** and **expected_keys = [Read baseline identities](table.pluck(list=`expected`, key="key"))** and **actual_keys = [Read proposed identities](table.pluck(list=`actual`, key="key"))**.
1. len(`expected`) != len(`actual`) or len(table.unique(list=`expected_keys`)) != len(`expected`) or len(table.unique(list=`actual_keys`)) != len(`actual`)
  Return *[Reject missing or repeated data identities](result.error(code="data.identities", message="baseline and actual must have equal counts and unique keys", operation="compare"))*.
- [Compare each source's actual native value](for expected as row)
  We **key = [Read its identity](access(`row`, "key"))** and **found = [Find the corresponding proposed value](table.find(list=`actual`, key="key", value=`key`))** and **same = False**.
  1. `found` != None
    We **same = [Compare original and proposed types and shapes](access(row, "value") == access(found, "value"))**.
  1. not `same`
    We **mismatched = [Retain the differing source identity](table.append(list=`mismatched`, item=`key`))**.
We **count = [Keep the actual baseline count](len(expected))**.

`evidence` =

| count | mismatched |
|---|---|
| `count` | `mismatched` |

1. len(`mismatched`) == 0
  Return *[Retain the unchanged data evidence](result.ok(value=`evidence`))*.
We **failed = [Keep the refactor incomplete](result.error(code="data.value_drift", message="inspect the mismatched source keys", operation="compare"))**.
Return *[Preserve differences on failure](table.put(in=`failed`, at="value", value=`evidence`))*.
