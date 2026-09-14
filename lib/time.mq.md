---
title: lib/time
description: English time wrappers
---

Unix time, format, parse, and sleep helpers.

## now_unix

Current Unix time in seconds.

Caller: [time.now_unix]

*> host_now_unix*

## now_ms

Current Unix time in milliseconds.

*> host_now_ms*

## format
    + `unix`
    + `pattern`

Format unix seconds with pattern.

*> host_format_time unix=`unix` pattern=`pattern`*

## sleep_ms
    + `ms`

Sleep for ms milliseconds.

*> host_sleep_ms ms=`ms`*

## parse
    + `text`
    + `pattern`

Parse text as time with pattern; returns unix seconds.

*> host_parse_time text=`text` pattern=`pattern`*
