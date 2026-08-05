---
title: lib/net — HTTP
description: Minimal HTTP client and URL encode
> lib/net.mq.md
---

# main

Import lib/net.mq.md. Functions: http_get(url) returns map status/body, http_post(url, body), url_encode(text). v1 speaks cleartext http only.

*`e` = > url_encode text=hello world *

> print text=`e`
