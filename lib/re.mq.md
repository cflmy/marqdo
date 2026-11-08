---
markup: occlusion
title: lib/re
description: Regular expressions (Mid M1). Unicode; no backreferences.
---

Regular expressions. Unicode; no backreferences. Prefer lib/text for literal replace.

Match Unicode regular expressions.

Behavior and limits: The engine has no backreferences. Current find functions return whole matches; capture groups and replacement callbacks are not promised. Literal replacement belongs in text.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/re.rs](../src/host/re.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## is_match

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | Regular expression supported by the regex host |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether text matches pattern |

Use `text`, `pattern` as the inputs for this operation.

Effects: pure.

Whether text matches pattern.

Caller: [re.is_match] text=`s` pattern="a+"

Return *[Whether text matches pattern](> host_re_is_match text=`text` pattern=`pattern`)*.

## find

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | Regular expression supported by the regex host |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | First match of pattern in text, or None |

Use `text`, `pattern` as the inputs for this operation.

Effects: pure.

First match of pattern in text, or None.

Return *[First match of pattern in text, or None](> host_re_find text=`text` pattern=`pattern`)*.

## find_all

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | Regular expression supported by the regex host |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | All matches of pattern in text |

Use `text`, `pattern` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/re.mq.md find_all 'text="a12 b34"' 'pattern="[0-9]+"' --effects none --json
```

All matches of pattern in text.

Return *[All matches of pattern in text](> host_re_find_all text=`text` pattern=`pattern`)*.

## replace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | Regular expression supported by the regex host |
| with | any | required | Replacement text applied to matched spans |
| count | any | None | Maximum replacement count; None uses the documented default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Replace matches of pattern in text with with; optional count limits replacements (None = all) |

Use `text`, `pattern`, `with`, `count`=None as the inputs for this operation.

Effects: pure.

Replace matches of pattern in text with with; optional count limits replacements (None = all).

Return *[Replace matches of pattern in text with with; optional count limits replacements (None = all)](> host_re_replace text=`text` pattern=`pattern` with=`with` count=`count`)*.

## split

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | Regular expression supported by the regex host |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Split text on pattern |

Use `text`, `pattern` as the inputs for this operation.

Effects: pure.

Split text on pattern.

Return *[Split text on pattern](> host_re_split text=`text` pattern=`pattern`)*.
