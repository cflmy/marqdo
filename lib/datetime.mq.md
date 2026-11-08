---
markup: occlusion
title: lib/datetime
description: Calendar moments as maps {unix, zone, iso} (Mid2 M7). Fixed offsets and embedded IANA timezone/DST rules.
---

Calendar moments as maps {unix, zone, iso} (Mid2 M7). Fixed offsets and embedded IANA timezone/DST rules.

Convert timestamps, IANA zones and local calendar dates.

Behavior and limits: zone remains the numeric offset; timezone preserves the IANA identity. add uses elapsed seconds; add_calendar preserves local wall time. DST gaps fail; folds require an explicit policy.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/datetime.rs](../src/host/datetime.rs)、[src/host/data_ops.rs](../src/host/data_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## now

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Read the clock and return the current UTC datetime record |

Effects: time.read.

Current UTC moment map.

Read the clock and return the current UTC datetime record.

Return *[Read the clock and return the current UTC datetime record](host_datetime_now())*.

## from_unix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| unix | number | required | Unix timestamp in seconds |
| zone | any | None | Fixed offset or named timezone accepted by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Convert Unix seconds to a datetime record in the selected zone; None means UTC |

Use `unix`, `zone`=None as the inputs for this operation.

Effects: pure.

Convert Unix seconds to a datetime record in the selected zone; None means UTC.

Return *[Convert Unix seconds to a datetime record in the selected zone; None means UTC](> host_datetime_from_unix unix=`unix` zone=`zone`)*.

## to_unix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dt | any | required | Datetime record returned by this library, not formatted text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Read unix seconds from a datetime record; formatting and timezone do not change the instant |

Use `dt` as the inputs for this operation.

Effects: pure.

Read unix seconds from a datetime record; formatting and timezone do not change the instant.

Return *[Read unix seconds from a datetime record; formatting and timezone do not change the instant](> host_datetime_to_unix dt=`dt`)*.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| pattern | any | None | None for default parsing, rfc3339 or a strftime format |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Parse RFC3339 or YYYY-MM-DD with optional time; an explicit pattern uses strftime or rfc3339 |

Use `text`, `pattern`=None as the inputs for this operation.

Effects: pure.

RFC3339 / `YYYY-MM-DD[ HH:MM:SS]` by default; `pattern=` is strftime or `rfc3339`.

Parse RFC3339 or YYYY-MM-DD with optional time; an explicit pattern uses strftime or rfc3339.

Return *[Parse RFC3339 or YYYY-MM-DD with optional time; an explicit pattern uses strftime or rfc3339](> host_datetime_parse text=`text` pattern=`pattern`)*.

## format

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dt | any | required | Datetime record returned by this library, not formatted text |
| style | any | None | Formatting style accepted by the host date formatter |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Format a datetime record as rfc3339, date, time, or an explicit strftime pattern |

Use `dt`, `style`=None as the inputs for this operation.

Effects: pure.

Styles: `rfc3339` (default), `date`, `time`, or a strftime pattern.

Format a datetime record as rfc3339, date, time, or an explicit strftime pattern.

Return *[Format a datetime record as rfc3339, date, time, or an explicit strftime pattern](> host_datetime_format dt=`dt` style=`style`)*.

## add

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dt | any | required | Datetime record returned by this library, not formatted text |
| days | any | None | Number of elapsed 24-hour days to add |
| hours | any | None | Number of elapsed hours to add |
| minutes | any | None | Number of elapsed minutes to add |
| seconds | any | None | Number of elapsed seconds to add |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Add elapsed days, hours, minutes and seconds; a day here is exactly 24 hours |

Use `dt`, `days`=None, `hours`=None, `minutes`=None, `seconds`=None as the inputs for this operation.

Effects: pure.

Add elapsed days, hours, minutes and seconds; a day here is exactly 24 hours. Named zones recompute their offset after the move.

Return *[Add elapsed days, hours, minutes and seconds; a day here is exactly 24 hours](> host_datetime_add dt=`dt` days=`days` hours=`hours` minutes=`minutes` seconds=`seconds`)*.

## in_zone

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dt | any | required | Datetime record returned by this library, not formatted text |
| zone | any | required | Fixed offset or named timezone accepted by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Re-express the same instant in a fixed offset or an IANA zone, retaining its Unix timestamp |

Use `dt`, `zone` as the inputs for this operation.

Effects: pure.

Re-express the same instant in `+08:00` / `UTC` / IANA zones like `Asia/Shanghai`.

Re-express the same instant in a fixed offset or an IANA zone, retaining its Unix timestamp.

Return *[Re-express the same instant in a fixed offset or an IANA zone, retaining its Unix timestamp](> host_datetime_in_zone dt=`dt` zone=`zone`)*.

## parse_local

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |
| zone | text | required | Fixed offset or named timezone accepted by this operation |
| ambiguity | text | "reject" | DST fold policy: reject, earliest or latest |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Parse YYYY-MM-DD HH:MM:SS in an IANA timezone as Result |

Use `text`, `zone`, `ambiguity`="reject" as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/datetime.mq.md parse_local 'text="2024-11-03 01:30:00"' 'zone="America/New_York"' 'ambiguity="earliest"' --effects none --json
```

Parse YYYY-MM-DD HH:MM:SS in an IANA timezone as Result. DST gaps fail; folds require reject/earliest/latest.

Return *[Parse YYYY-MM-DD HH:MM:SS in an IANA timezone as Result](> host_datetime_local text=`text` zone=`zone` ambiguity=`ambiguity`)*.

## add_calendar

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dt | map | required | Datetime record returned by this library, not formatted text |
| days | number | required | Local calendar days to add, preserving wall time across DST |
| ambiguity | text | "reject" | DST fold policy: reject, earliest or latest |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Add local calendar days as Result, preserving wall-clock time across DST |

Use `dt`, `days`, `ambiguity`="reject" as the inputs for this operation.

Effects: pure.

Add local calendar days as Result, preserving wall-clock time across DST. Legacy add uses elapsed duration: one day is 24 hours.

Return *[Add local calendar days as Result, preserving wall-clock time across DST](> host_datetime_calendar_add dt=`dt` days=`days` ambiguity=`ambiguity`)*.
