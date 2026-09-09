---
title: lib/datetime
description: Calendar moments as maps {unix, zone, iso} (Mid2 M7). Fixed offsets + a few IANA aliases.
---

## now

Current UTC moment map.

**> host_datetime_now**

## from_unix
    + `unix`
    + `zone`=None

**> host_datetime_from_unix unix=`unix` zone=`zone`**

## to_unix
    + `dt`

**> host_datetime_to_unix dt=`dt`**

## parse
    + `text`
    + `pattern`=None

RFC3339 / `YYYY-MM-DD[ HH:MM:SS]` by default; `pattern=` is strftime or `rfc3339`.

**> host_datetime_parse text=`text` pattern=`pattern`**

## format
    + `dt`
    + `style`=None

Styles: `rfc3339` (default), `date`, `time`, or a strftime pattern.

**> host_datetime_format dt=`dt` style=`style`**

## add
    + `dt`
    + `days`=None
    + `hours`=None
    + `minutes`=None
    + `seconds`=None

**> host_datetime_add dt=`dt` days=`days` hours=`hours` minutes=`minutes` seconds=`seconds`**

## in_zone
    + `dt`
    + `zone`

Re-express the same instant in `+08:00` / `UTC` / fixed aliases like `Asia/Shanghai`.

**> host_datetime_in_zone dt=`dt` zone=`zone`**
