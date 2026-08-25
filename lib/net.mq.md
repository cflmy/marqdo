---
title: lib/net
description: English HTTP(S) wrappers. Optional headers / content_type via host aliases.
---

## http_get
    + `url`
    + `headers`=None

HTTPS supported. Optional named arg `headers=` (map).

**> host_http_get url=`url` headers=`headers`**

## http_post
    + `url`
    + `body`
    + `content_type`=None
    + `headers`=None

Default content type is JSON. Optional `content_type=` / `headers=` (map).

**> host_http_post url=`url` body=`body` content_type=`content_type` headers=`headers`**

## http_post_sse
    + `url`
    + `body`
    + `content_type`=None
    + `headers`=None
    + `echo`=False

POST and consume OpenAI-compatible SSE. Returns `{status, events}` where `events` is a list of `{type, …}` maps (`delta` / `done` / `error`). Optional `echo=True` prints delta text to stdout as chunks arrive.

**> host_http_post_sse url=`url` body=`body` content_type=`content_type` headers=`headers` echo=`echo`**

## openai_sse_parse
    + `text`
    + `echo`=False

Offline: parse an OpenAI chat SSE body into the same event list (no network).

**> host_openai_sse_parse text=`text` echo=`echo`**

## http_request
    + `method`
    + `url`
    + `body`=None
    + `content_type`=None
    + `headers`=None

Optional `body=` / `content_type=` / `headers=`.

**> host_http_request method=`method` url=`url` body=`body` content_type=`content_type` headers=`headers`**

## url_encode
    + `text`

**> host_url_encode text=`text`**

## cookie_parse
    + `text`
    + `is_response`=False

Parse a `Cookie` request header (default) or one or more `Set-Cookie` response headers (`is_response=True`) into a list of `{name, value, path, domain, expires, max_age, secure, http_only, same_site}`.

**> host_cookie_parse text=`text` is_response=`is_response`**

## multipart_parse
    + `body`
    + `boundary`

Parse a `multipart/form-data` request body (given its `boundary`) into a list of `{name, filename?, content_type?, value}` parts.

**> host_multipart_parse body=`body` boundary=`boundary`**
