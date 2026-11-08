---
markup: occlusion
title: lib/text
description: Text helpers (Mid M4). Literal replace — use lib/re for regex.
---

Small text helpers. Prefer these over hand-rolled splits; regex belongs in lib/re.

Clean and combine Unicode text.

Behavior and limits: Use re for patterns and bytes for arbitrary binary. split and join use exact separators; no implicit document parsing.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/text_ops.rs](../src/host/text_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## str_trim

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| s | text | required | Input text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Trim ends of string s |

Use `s` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/text.mq.md str_trim 's="  readable text  "' --effects none --json
```

Trim ends of string s.

Return *[Trim](trim(`s`))*.

## str_split

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| s | text | required | Input text |
| sep | text | required | Text separator used for splitting or joining |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Split string s on separator sep |

Use `s`, `sep` as the inputs for this operation.

Effects: pure.

Split string s on separator sep.

Return *[Split](split(value=`s`, sep=`sep`))*.

## str_join

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |
| sep | text | required | Text separator used for splitting or joining |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Join list xs with separator sep |

Use `xs`, `sep` as the inputs for this operation.

Effects: pure.

Join list xs with separator sep.

Return *[Join](join(value=`xs`, sep=`sep`))*.

## contains

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| sub | any | required | Substring to search for |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether text contains substring sub |

Use `text`, `sub` as the inputs for this operation.

Effects: pure.

Whether text contains substring sub.

Caller: [text.contains] text="hello" sub="ell"

Return *[Whether text contains substring sub](> host_text_contains text=`text` sub=`sub`)*.

## starts_with

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| prefix | text | required | Prefix added to the generated name or tested against text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether text starts with prefix |

Use `text`, `prefix` as the inputs for this operation.

Effects: pure.

Whether text starts with prefix.

Return *[Whether text starts with prefix](> host_text_starts_with text=`text` prefix=`prefix`)*.

## ends_with

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| suffix | any | required | Text suffix to test |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether text ends with suffix |

Use `text`, `suffix` as the inputs for this operation.

Effects: pure.

Whether text ends with suffix.

Return *[Whether text ends with suffix](> host_text_ends_with text=`text` suffix=`suffix`)*.

## replace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| old | any | required | Exact substring to replace |
| new | any | required | Replacement text |
| count | any | None | Maximum replacement count; None uses the documented default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Literal replace in text: replace old with new, optional count |

Use `text`, `old`, `new`, `count`=None as the inputs for this operation.

Effects: pure.

Literal replace in text: replace old with new, optional count.

Return *[Literal replace in text: replace old with new, optional count](> host_text_replace text=`text` old=`old` new=`new` count=`count`)*.

## to_upper

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Uppercase text |

Use `text` as the inputs for this operation.

Effects: pure.

Uppercase text.

Return *[Uppercase text](> host_text_to_upper text=`text`)*.

## to_lower

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Lowercase text |

Use `text` as the inputs for this operation.

Effects: pure.

Lowercase text.

Return *[Lowercase text](> host_text_to_lower text=`text`)*.

## repeat

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| n | number | required | Number of repetitions |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Repeat text n times |

Use `text`, `n` as the inputs for this operation.

Effects: pure.

Repeat text n times.

Return *[Repeat text n times](> host_text_repeat text=`text` n=`n`)*.

## pad

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| width | number | required | Target padded character width |
| fill | any | None | Single padding character |
| align | any | None | Padding alignment: left, right or center |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Pad text to width with fill and align (left / right / center) |

Use `text`, `width`, `fill`=None, `align`=None as the inputs for this operation.

Effects: pure.

Pad text to width with fill and align (left / right / center).

Return *[Pad text to width with fill and align (left / right / center)](> host_text_pad text=`text` width=`width` fill=`fill` align=`align`)*.
