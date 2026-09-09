---
title: lib/url
description: Parse URLs and query strings (Mid2 M7). Segment encoding stays in net.url_encode.
---

## parse
    + `text`

Returns `{scheme, userinfo, host, port, path, query, fragment}`.

**> host_url_parse text=`text`**

## query_parse
    + `text`

Parse `a=1&b=2` into a map; repeated keys become a list.

**> host_url_query_parse text=`text`**

## query_stringify
    + `map`

**> host_url_query_stringify map=`map`**
