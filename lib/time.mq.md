---
markup: occlusion
title: lib/time
description: English time wrappers
---

Unix time, format, parse, and sleep helpers.

Format Unix time and control elapsed waits.

Behavior and limits: now reads the clock; sleep controls execution and is clamped in capture. Calendar arithmetic and IANA rules live in datetime.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/time.rs](../src/host/time.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## now_unix

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Current Unix time in seconds |

Effects: time.read.

Current Unix time in seconds.

Caller: [time.now_unix]

Return *[Current Unix time in seconds](host_now_unix())*.

## now_ms

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Current Unix time in milliseconds |

Effects: time.read.

Current Unix time in milliseconds.

Return *[Current Unix time in milliseconds](host_now_ms())*.

## format

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| unix | number | required | Unix timestamp in seconds |
| pattern | text | required | strftime date/time format string |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Format unix seconds with pattern |

Use `unix`, `pattern` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/time.mq.md format unix=0 'pattern="%Y-%m-%d"' --effects none --json
```

Format unix seconds with pattern.

Return *[Format unix seconds with pattern](> host_format_time unix=`unix` pattern=`pattern`)*.

## sleep_ms

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ms | any | required | Duration or delay in milliseconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Sleep for ms milliseconds |

Use `ms` as the inputs for this operation.

Effects: process.control.

Sleep for ms milliseconds.

Return *[Sleep for ms milliseconds](> host_sleep_ms ms=`ms`)*.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | text | required | strftime date/time format string |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Parse text as time with pattern; returns unix seconds |

Use `text`, `pattern` as the inputs for this operation.

Effects: pure.

Parse text as time with pattern; returns unix seconds.

Return *[Parse text as time with pattern; returns unix seconds](> host_parse_time text=`text` pattern=`pattern`)*.
