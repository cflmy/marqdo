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
