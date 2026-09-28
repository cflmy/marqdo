---
title: ext/data/storage
description: Object storage Resource (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/data: native web plugin not found
  > sys.exit code=1
****

# storage
    + `url`="file:data/blobs"

Object storage. `file:dir` stores blobs on disk (offline / gold). `s3://bucket?endpoint=http://127.0.0.1:9000&access_key=…&secret_key=…` talks to MinIO / S3.

> _load
*> web_storage_new url=`url`*

## put
    + `key`
    + `body`=None
    + `path`=None
    + `content_type`="application/octet-stream"

Provide either `body` (text) or `path` (local file to upload).

**url = [url](self)**
*> web_storage_put url=`url` key=`key` body=`body` path=`path` content_type=`content_type`*

## get
    + `key`

**url = [url](self)**
*> web_storage_get url=`url` key=`key`*

## delete
    + `key`

**url = [url](self)**
*> web_storage_delete url=`url` key=`key`*

## list
    + `prefix`=""

**url = [url](self)**
*> web_storage_list url=`url` prefix=`prefix`*

# media
    + `storage`=None

Offline helpers for upload validation and saving into `# storage` (also used by HTTP `app.upload`).

> _load
*> web_media_new storage=`storage`*

## validate
    + `filename`
    + `content_type`="application/octet-stream"
    + `size`
    + `max_bytes`=5242880
    + `types`=None

*> web_upload_validate filename=`filename` content_type=`content_type` size=`size` max_bytes=`max_bytes` types=`types`*

## save
    + `path`
    + `key`=None
    + `content_type`="application/octet-stream"
    + `prefix`=uploads/
    + `storage`=None

**st = storage**
1. `st` == None
  **st = [storage](self)**
2. *

*> web_upload_save storage=`st` path=`path` key=`key` content_type=`content_type` prefix=`prefix`*

