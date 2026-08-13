---
title: lib/net — HTTP(S)
description: Minimal HTTP(S) client and URL encode
import net:lib/net.mq.md
---

# main

Import lib/net.mq.md. Functions: http_get, http_post, http_post_sse, openai_sse_parse, http_request, url_encode. HTTPS and optional headers= map are supported. Default post content-type is JSON. `http_post_sse` / `openai_sse_parse` return OpenAI chat stream event lists (`delta` / `done`).

*`e` = > net.url_encode text=hello world *

> print text=`e`
