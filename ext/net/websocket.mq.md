---
title: ext/net/websocket
description: WebSocket client Resource (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/net: native web plugin not found
  > sys.exit code=1
****

# ws
    + `timeout_sec`=30

WebSocket client helper.

> _load
`out` =

| timeout_sec | _type |
|-------------|-------|
| `timeout_sec` | ws |

*out*

## connect
    + `url`
    + `message`=""
    + `headers`=None

Single request–response: connect to `url`, send `message`, collect up to 256 server text replies (each up to 1 MiB), close. Transport errors and timeouts return `{ok:false, error}` instead of looking like an empty successful reply.

**timeout = [timeout_sec](self)**
*> web_ws_connect url=`url` message=`message` headers=`headers` timeout_sec=`timeout`*
