---
title: Mid stdlib — datetime and url
description: Calendar moments + URL/query parse (Mid2 M7)
import dt:lib/datetime.mq.md
import url:lib/url.mq.md
---

# Time and addresses without plugins

Moments are maps `{unix, zone, iso}`. Parse HTTPS URLs and round-trip query strings in `lib/url` (segment encoding stays in `net.url_encode`).

# main

*epoch = > dt.from_unix unix=0 zone="Asia/Shanghai"*
> print text=`epoch`[^iso]

*u = > url.parse text="https://example.com/path?q=1#top"*
> print text=`u`[^host]
> print text=`u`[^query]
