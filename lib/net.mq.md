---
markup: occlusion
import:
  table: lib/table.mq.md
  text: lib/text.mq.md
title: lib/net
description: English HTTP(S) wrappers. Optional headers / content_type via host aliases.
---

English HTTP(S) wrappers. Optional headers / content_type via host aliases.

Use HTTP, response parsing and bounded downloads.

Behavior and limits: get requires network.read; configurable request conservatively requires network.write because its method is dynamic. request/get return Result with response headers; legacy http_* retain their historical contracts. Downloads also require fs.write.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/net.rs](../src/host/net.rs)、[src/host/net_ops.rs](../src/host/net_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## http_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| headers | any | None | HTTP header name/value record; None sends no extra headers |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | HTTPS supported |

Use `url`, `headers`=None as the inputs for this operation.

Effects: network.read.

HTTPS supported. Optional named arg `headers=` (map). Responses are capped at 8 MiB before decoding, so an upstream cannot exhaust the interpreter's memory.

Return *[HTTPS supported](> host_http_get url=`url` headers=`headers`)*.

## http_post

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| body | any | required | Request body, or replacement text for document operations |
| content_type | any | None | HTTP content type override; None uses the helper default |
| headers | any | None | HTTP header name/value record; None sends no extra headers |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Default content type is JSON |

Use `url`, `body`, `content_type`=None, `headers`=None as the inputs for this operation.

Effects: network.write.

Default content type is JSON. Optional `content_type=` / `headers=` (map).

Return *[Default content type is JSON](> host_http_post url=`url` body=`body` content_type=`content_type` headers=`headers`)*.

## http_post_sse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| body | any | required | Request body, or replacement text for document operations |
| content_type | any | None | HTTP content type override; None uses the helper default |
| headers | any | None | HTTP header name/value record; None sends no extra headers |
| echo | bool | False | Whether to print streamed SSE text as it arrives |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | POST and consume OpenAI-compatible SSE |

Use `url`, `body`, `content_type`=None, `headers`=None, `echo`=False as the inputs for this operation.

Effects: network.write.

POST and consume OpenAI-compatible SSE. Returns `{status, events}` where `events` is a list of `{type, …}` maps (`delta` / `done` / `error`). Optional `echo=True` prints delta text to stdout as chunks arrive.

Return *[POST and consume OpenAI-compatible SSE](> host_http_post_sse url=`url` body=`body` content_type=`content_type` headers=`headers` echo=`echo`)*.

## openai_sse_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| echo | bool | False | Whether to print streamed SSE text as it arrives |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Offline: parse an OpenAI chat SSE body into the same event list (no network) |

Use `text`, `echo`=False as the inputs for this operation.

Effects: pure.

Offline: parse an OpenAI chat SSE body into the same event list (no network).

Return *[Offline: parse an OpenAI chat SSE body into the same event list (no network)](> host_openai_sse_parse text=`text` echo=`echo`)*.

## http_request

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| method | text | required | HTTP method, for example GET or POST |
| url | text | required | Target URL; HTTP helpers accept http or https |
| body | any | None | Request body, or replacement text for document operations |
| content_type | any | None | HTTP content type override; None uses the helper default |
| headers | any | None | HTTP header name/value record; None sends no extra headers |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Optional `body=` / `content_type=` / `headers=` |

Use `method`, `url`, `body`=None, `content_type`=None, `headers`=None as the inputs for this operation.

Effects: network.write.

Optional `body=` / `content_type=` / `headers=`.

Return *[Optional body= / content_type= / headers=](> host_http_request method=`method` url=`url` body=`body` content_type=`content_type` headers=`headers`)*.

## url_encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Percent-encode a URL segment |

Use `text` as the inputs for this operation.

Effects: pure.

Percent-encode a URL segment. Spaces become `%20` (not form/query `+`), so literal `+` remains unambiguous in paths.

Return *[Percent-encode a URL segment](> host_url_encode text=`text`)*.

## cookie_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |
| is_response | any | False | Whether to parse a response header block rather than a request |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Parse a `Cookie` request header (default) or one or more `Set-Cookie` response headers (`is_response=True`) into a list of `{name, value, path, domain, expires, max_age, secure, http_only, same_site}` |

Use `text`, `is_response`=False as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/net.mq.md cookie_parse 'text="a=1; b=2"' --effects none --json
```

Parse a `Cookie` request header (default) or one or more `Set-Cookie` response headers (`is_response=True`) into a list of `{name, value, path, domain, expires, max_age, secure, http_only, same_site}`.

Return *[Parse a Cookie request header (default) or one or more Set-Cookie response headers (is_response=True) into a list of {name, value, path, domain, expires, max_age, secure,](> host_cookie_parse text=`text` is_response=`is_response`)*.

## multipart_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| body | any | required | Request body, or replacement text for document operations |
| boundary | any | required | Multipart delimiter from the Content-Type header |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Parse a `multipart/form-data` request body (given its `boundary`) into a list of `{name, filename?, content_type?, value}` parts |

Use `body`, `boundary` as the inputs for this operation.

Effects: pure.

Parse a `multipart/form-data` request body (given its `boundary`) into a list of `{name, filename?, content_type?, value}` parts.

Return *[Parse a multipart/form-data request body (given its boundary) into a list of {name, filename?, content_type?, value} parts](> host_multipart_parse body=`body` boundary=`boundary`)*.

## markdown_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Parse Markdown / GFM into HTML (pure transform, no network I/O) |

Use `text` as the inputs for this operation.

Effects: pure.

Parse Markdown / GFM into HTML (pure transform, no network I/O).

Return *[Parse Markdown / GFM into HTML (pure transform, no network I/O)](> host_markdown_parse text=`text`)*.

## request

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| method | text | required | HTTP method, for example GET or POST |
| url | text | required | Target URL; HTTP helpers accept http or https |
| body | any | None | Request body, or replacement text for document operations |
| headers | any | None | HTTP header name/value record; None sends no extra headers |
| timeout_ms | number | 30000 | Timeout in milliseconds; inspect the operation result after waiting |
| max_bytes | number | 8388608 | Maximum bytes read or expanded; exceeding it fails |
| response | text | "text" | Response body representation: text or bytes |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Return Result; value has status, headers and body |

Use `method`, `url`, `body`=None, `headers`=None, `timeout_ms`=30000, `max_bytes`=8388608, `response`="text" as the inputs for this operation.

Effects: network.write.

Body is text, a bytes record or None; serialize JSON with json.stringify first. Request headers are a map; response headers are name/values records. Output options and destinations are checked before network IO.

Return Result; value has status, headers and body. Timeout is milliseconds; response is text or bytes; headers are name/values records. HTTP error status is a valid response; transport failure is an error.

Return *[Body is text, a bytes record or None; serialize JSON with json.stringify first](> host_http_request_result method=`method` url=`url` body=`body` headers=`headers` timeout_ms=`timeout_ms` max_bytes=`max_bytes` response=`response`)*.

## download

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| path | text | required | Path within the granted filesystem root |
| headers | any | None | HTTP header name/value record; None sends no extra headers |
| timeout_ms | number | 30000 | Timeout in milliseconds; inspect the operation result after waiting |
| max_bytes | number | 67108864 | Maximum bytes read or expanded; exceeding it fails |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Bounded streaming download; atomically replace destination only on successful status and complete transfer |

Use `url`, `path`, `headers`=None, `timeout_ms`=30000, `max_bytes`=67108864 as the inputs for this operation.

Effects: fs.write,network.read.

Bounded streaming download; atomically replace destination only on successful status and complete transfer. Result value has status/headers/size/path; requires network.read and fs.write.

Return *[Bounded streaming download; atomically replace destination only on successful status and complete transfer](> host_http_download url=`url` path=`path` headers=`headers` timeout_ms=`timeout_ms` max_bytes=`max_bytes`)*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| headers | any | None | HTTP header name/value record; None sends no extra headers |
| timeout_ms | number | 30000 | Timeout in milliseconds; inspect the operation result after waiting |
| max_bytes | number | 8388608 | Maximum bytes read or expanded; exceeding it fails |
| response | text | "text" | Response body representation: text or bytes |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Read-only GET as Result, requiring only network |

Use `url`, `headers`=None, `timeout_ms`=30000, `max_bytes`=8388608, `response`="text" as the inputs for this operation.

Effects: network.read.

Read-only GET as Result, requiring only network.read. Headers are name/values records; response is text or bytes.

Return *[Read-only GET as Result, requiring only network.read](> host_http_get_result url=`url` headers=`headers` timeout_ms=`timeout_ms` max_bytes=`max_bytes` response=`response`)*.

## header_values

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| headers | list | required | headers records from a response |
| name | text | required | Header name |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | All values for the header |

Use `headers`, `name` as the inputs for this operation.

Effects: pure.

Return all values for a case-insensitive response header; missing yields an empty list. Request headers remain a map; response headers use records to preserve arbitrary names and repeated values.

We retain **`lower` = [To lower](text.to_lower(text=`name`))**.
We retain **`matches` = [Where](table.where(list=`headers`, key="name", value=`lower`))**.
We retain **`values` = [Pluck](table.pluck(list=`matches`, key="values"))**.
Return *[Flatten](table.flatten(list=`values`))*.
