---
markup: occlusion
title: lib/html
description: HTML escape / unescape (Mid2 M8). XSS floor for string assembly.
---

HTML escape helpers. Use before stitching untrusted text into markup.

Escape or unescape HTML text.

Behavior and limits: Escaping text does not sanitize an arbitrary HTML document. Build web content as Markdown leaves and structured View nodes.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/html_ops.rs](../src/host/html_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## escape

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Escape text for HTML |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/html.mq.md escape 'text="<strong>&"' --effects none --json
```

Escape text for HTML.

Caller: [html.escape] text=`s`

Return *[Escape text for HTML](> host_html_escape text=`text`)*.

## unescape

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Unescape HTML entities in text |

Use `text` as the inputs for this operation.

Effects: pure.

Unescape HTML entities in text.

Return *[Unescape HTML entities in text](> host_html_unescape text=`text`)*.
