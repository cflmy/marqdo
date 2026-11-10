---
markup: occlusion
title: Network resources — URL data and WebSocket exchanges
import:
  url_api: ext/net/url.mq.md
  socket_api: ext/net/websocket.mq.md
---

This package provides [URL encoding](url.mq.md), the small [HTTP result constructor](http.mq.md),
and a native [WebSocket request/response resource](websocket.mq.md). For HTTP clients with
status, timeout, response limits and downloads, read the standard library net.mq.md directly.
The HTTP leaf here constructs a result record; its name does not imply an implemented HTTP client.
绑定服务端运输使用 websocket.route；中文入口是[实时绑定](实时.mq.md)，它与一次客户端 exchange 的生命周期不同。

URL encoding is pure and needs no plugin. This command is ready to run from the repository root:

```sh
marqdo call ext/net/index.mq.md encode 'text="Marqdo notes/one"' --json
```

WebSocket exchanges need the native web plugin and an explicitly chosen server. Build the plugin
with scripts/build-web-plugin.sh, then supply a controlled endpoint to exchange. No example opens
a public socket implicitly. The resource closes after one bounded exchange; it is not a persistent
subscription API. Read the returned ok/error as well as the reply data before using the outcome.

Develop URL transformations by composing the public standard library; native transport belongs in
the web plugin. Preserve timeout units, byte/reply caps and close behavior. Add controlled local
server tests for transport failures and timeouts instead of treating an empty reply as success.

## encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text component to percent-encode; this does not fetch a URL |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Encoded URL component returned by the public encoding helper |

The input `text` is content for an encoded component, not an already assembled URL to navigate.
Keeping this transformation separate makes it possible to build request data without network
permission. We return *[Percent-encode the selected component](url_api.encode(text=`text`))*
so the caller can place it in the intended path or query according to its own URL structure.

## exchange

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Explicit ws/wss endpoint controlled by the caller |
| message | text | "" | Single outgoing text message |
| timeout_sec | number | 30 | Positive exchange timeout in seconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native exchange result; failures carry ok=false/error instead of an empty successful reply |

The caller supplies `url`, optionally `message`="" and `timeout_sec`=30. This tool performs a
real exchange, so it needs plugin permission and an available endpoint. The native leaf caps
replies and frame sizes and closes the connection before returning; it does not retry a send.

We create **socket = [Configure a bounded WebSocket receiver](socket_api.ws(timeout_sec=`timeout_sec`))**,
then return *[Send the message and collect the bounded reply](> `socket`.connect url=`url` message=`message` )*.
Inspect the actual result before deciding whether a follow-up request is appropriate.
