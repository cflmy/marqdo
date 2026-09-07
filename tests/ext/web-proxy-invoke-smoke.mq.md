---
title: web proxy/invoke smoke
description: Offline assemble of app.proxy and app.invoke routes (no listen).
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

`proxy_tbl` =

| path | upstream | stream | strip_prefix | methods | headers_from_env | timeout_ms |
|------|----------|--------|--------------|---------|------------------|------------|
| /proxy/chat | $OPENAI_BASE_URL | True | /proxy | POST | OPENAI_API_KEY=Authorization | 60000 |

`invoke_tbl` =

| path | method | fn | body | return |
|------|--------|-----|------|--------|
| /api/echo | POST | demo.echo | json | json |

*pg = > web.page title="proxy-invoke"*
*app = > web.app page=`pg`*
*app = > `app`.configure proxy=`proxy_tbl` invoke=`invoke_tbl`*
*app = > `app`.proxy path="/v1/stream" upstream="https://example.com/v1/chat" stream=True headers_from_env="TOKEN=X-Token"*
*app = > `app`.invoke path="/api/ping" fn="demo.ping" method="GET" body="query"*

*proutes = app[^proxy_routes]*
*chat = proutes[^proxy/chat]*
*up = chat[^upstream]*
1. `up` == "$OPENAI_BASE_URL"
  > print text=proxy-table-ok
2. *
  > print text=proxy-table-fail

*stream = chat[^stream]*
1. `stream`
  > print text=proxy-stream-ok
2. *
  > print text=proxy-stream-fail

*hdr = chat[^headers_from_env]*
1. `hdr` == "OPENAI_API_KEY=Authorization"
  > print text=proxy-env-ok
2. *
  > print text=proxy-env-fail

*v1 = proutes[^v1/stream]*
*v1u = v1[^upstream]*
1. `v1u` == "https://example.com/v1/chat"
  > print text=proxy-method-ok
2. *
  > print text=proxy-method-fail

*iroutes = app[^invoke_routes]*
*echo = iroutes[^api/echo]*
*efn = echo[^fn]*
1. `efn` == "demo.echo"
  > print text=invoke-table-ok
2. *
  > print text=invoke-table-fail

*ping = iroutes[^api/ping]*
*pm = ping[^method]*
1. `pm` == "GET"
  > print text=invoke-method-ok
2. *
  > print text=invoke-method-fail
