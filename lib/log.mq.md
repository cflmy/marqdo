---
markup: occlusion
title: lib/log
description: Level-filtered one-line logs (Mid M6 + Mid2 M10 fields). Default level info.
---

Level-filtered one-line logs (Mid M6 + Mid2 M10 fields). Default level info.

Emit level-filtered diagnostic lines.

Behavior and limits: set_level changes this run only. Logs are side effects, not return values. call captures stdout separately from its JSON result.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/log.rs](../src/host/log.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## set_level

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| level | any | required | Log threshold: debug, info, warn or error |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | level is debug / info / warn / error |

Use `level` as the inputs for this operation.

Effects: pure.

level is debug / info / warn / error.

Return *[level is debug / info / warn / error](> host_log_set_level level=`level`)*.

## debug

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| fields | any | None | Structured field/value record attached to the log event |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Emit a debug log line; optional fields map |

Use `text`, `fields`=None as the inputs for this operation.

Effects: output.write.

Emit a debug log line; optional fields map.

We retain **`line` = [Log line]([Emit a debug log line; optional fields map](host_log_line(level="debug", text=`text`, fields=`fields`)))**.
1. `line`
  We **[Print the prepared message](print(text=`line`))**.
2. *
  Return *None*.

## info

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| fields | any | None | Structured field/value record attached to the log event |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Emit an info log line; optional fields map |

Use `text`, `fields`=None as the inputs for this operation.

Effects: output.write.

Call this function directly:

```sh
marqdo call lib/log.mq.md info 'text="handbook example"' 'fields={"task":"sample"}' --effects output.write --json
```

Emit an info log line; optional fields map.

We retain **`line` = [Log line]([Emit an info log line; optional fields map](host_log_line(level="info", text=`text`, fields=`fields`)))**.
1. `line`
  We **[Print the prepared message](print(text=`line`))**.
2. *
  Return *None*.

## warn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| fields | any | None | Structured field/value record attached to the log event |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Emit a warn log line; optional fields map |

Use `text`, `fields`=None as the inputs for this operation.

Effects: output.write.

Emit a warn log line; optional fields map.

We retain **`line` = [Log line]([Emit a warn log line; optional fields map](host_log_line(level="warn", text=`text`, fields=`fields`)))**.
1. `line`
  We **[Print the prepared message](print(text=`line`))**.
2. *
  Return *None*.

## error

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| fields | any | None | Structured field/value record attached to the log event |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Emit an error log line; optional fields map |

Use `text`, `fields`=None as the inputs for this operation.

Effects: output.write.

Emit an error log line; optional fields map.

We retain **`line` = [Log line]([Emit an error log line; optional fields map](host_log_line(level="error", text=`text`, fields=`fields`)))**.
1. `line`
  We **[Print the prepared message](print(text=`line`))**.
2. *
  Return *None*.
