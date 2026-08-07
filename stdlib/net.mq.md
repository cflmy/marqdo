---
title: lib/net — HTTP(S)
description: Minimal HTTP(S) client and URL encode
> lib/net.mq.md
---

# main

Import lib/net.mq.md. Functions: http_get(url), http_post(url, body), http_request(method, url), url_encode(text). HTTPS and optional headers= map are supported. Default post content-type is JSON.

*`e` = > url_encode text=hello world *

> print text=`e`
