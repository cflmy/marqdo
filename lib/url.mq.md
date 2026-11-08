---
markup: occlusion
title: lib/url
description: Parse URLs and query strings (Mid2 M7). Segment encoding stays in net.url_encode.
---

Parse URLs and query strings. Segment encoding stays in net.url_encode.

Parse addresses and repeated query parameters.

Behavior and limits: Repeated query keys become lists; do not assume every map value is scalar. URL encoding is distinct from HTML escaping.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/url_ops.rs](../src/host/url_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Parse URL text into scheme, userinfo, host, port, path, query, fragment |

Use `text` as the inputs for this operation.

Effects: pure.

Parse URL text into scheme, userinfo, host, port, path, query, fragment.

Caller: [url.parse] text=`u`

Return *[Parse URL text into scheme, userinfo, host, port, path, query, fragment](> host_url_parse text=`text`)*.

## query_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Parse a=1&b=2 into a map; repeated keys become a list |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/url.mq.md query_parse 'text="tag=a&tag=b&name=Marqdo"' --effects none --json
```

Parse a=1&b=2 into a map; repeated keys become a list.

Return *[Parse a=1&b=2 into a map; repeated keys become a list](> host_url_query_parse text=`text`)*.

## query_stringify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | map | required | Input string-keyed map |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Build a query string from map |

Use `map` as the inputs for this operation.

Effects: pure.

Build a query string from map.

Return *[Build a query string from map](> host_url_query_stringify map=`map`)*.
