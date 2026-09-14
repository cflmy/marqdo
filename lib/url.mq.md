---
title: lib/url
description: Parse URLs and query strings (Mid2 M7). Segment encoding stays in net.url_encode.
---

Parse URLs and query strings. Segment encoding stays in net.url_encode.

## parse
    + `text`

Parse URL text into scheme, userinfo, host, port, path, query, fragment.

Caller: [url.parse] text=`u`

*> host_url_parse text=`text`*

## query_parse
    + `text`

Parse a=1&b=2 into a map; repeated keys become a list.

*> host_url_query_parse text=`text`*

## query_stringify
    + `map`

Build a query string from map.

*> host_url_query_stringify map=`map`*
