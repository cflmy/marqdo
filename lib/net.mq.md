---
title: lib/net
description: English HTTP(S) wrappers. Optional headers / content_type via host aliases.
---

## http_get
    + `url`

HTTPS supported. Optional named arg `headers=` (map).

**> host_http_get url=`url`**

## http_post
    + `url`
    + `body`

Default content type is JSON. Optional `content_type=` / `headers=` (map).

**> host_http_post url=`url` body=`body`**

## http_request
    + `method`
    + `url`

Optional `body=` / `content_type=` / `headers=`.

**> host_http_request method=`method` url=`url`**

## url_encode
    + `text`

**> host_url_encode text=`text`**
