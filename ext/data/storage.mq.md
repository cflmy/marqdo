---
markup: occlusion
title: ext/data/storage
description: Object storage Resource (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

Use this resource for explicitly configured storage URLs; use lib/fs.mq.md for ordinary rooted filesystem operations. [The data entry](index.mq.md) explains resource ownership and the native web dependency. File bodies and byte encodings follow the native storage contract, so callers must inspect the returned operation result before publishing a resource URL. Add backend support in the existing plugin rather than encoding transport logic in callers.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/data: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

# storage

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | "file:data/blobs" | Resource connection URL; scheme selects the documented backend |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Object storage. `file:dir` stores blobs on disk (offline / gold). `s3://bucket?endpoint=http://127.0.0.1:9000&access_key=…&secret_key=…` talks to MinIO / S3. |

Use `url`="file:data/blobs" as the inputs for this operation.

Object storage. `file:dir` stores blobs on disk (offline / gold). `s3://bucket?endpoint=http://127.0.0.1:9000&access_key=…&secret_key=…` talks to MinIO / S3.

We **[Load the native plugin required by this operation](_load())**.
Return *[Object storage. file:dir stores blobs on disk (offline / gold). s3://bucket?endpoint=http://127.0.0.1:9000&access_key=…&secret_key=… talks to MinIO / S3.](web_storage_new(url=`url`))*.

## put

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |
| body | any | None | Object or request content, as described by the operation |
| path | any | None | Explicit filesystem path; honor the documented filesystem root and operation effects |
| content_type | any | "application/octet-stream" | MIME type associated with the supplied content |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Provide either `body` (text) or `path` (local file to upload). |

Use `key`, `body`=None, `path`=None, `content_type`="application/octet-stream" as the inputs for this operation.

Provide either `body` (text) or `path` (local file to upload).

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Provide either body (text) or path (local file to upload).](web_storage_put(url=`url`, key=`key`, body=`body`, path=`path`, content_type=`content_type`))*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read the object selected by its storage key and retain the backend result shape. |

Use `key` as the inputs for this operation.

Read the object selected by its storage key and retain the backend result shape.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Read the object selected by its storage key and retain the backend result shape.](web_storage_get(url=`url`, key=`key`))*.

## delete

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | any | required | Entry or field key within the selected resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delete the selected object from this storage resource. |

Use `key` as the inputs for this operation.

Delete the selected object from this storage resource.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Delete the selected object from this storage resource.](web_storage_delete(url=`url`, key=`key`))*.

## list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| prefix | any | "" | Object-key or generated-name prefix |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List object entries under the supplied key prefix. |

Use `prefix`="" as the inputs for this operation.

List object entries under the supplied key prefix.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[List object entries under the supplied key prefix.](web_storage_list(url=`url`, prefix=`prefix`))*.

# media

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| storage | any | None | Storage resource created by the public storage constructor |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Offline helpers for upload validation and saving into `# storage` (also used by HTTP `app.upload`). |

Use `storage`=None as the inputs for this operation.

Offline helpers for upload validation and saving into `# storage` (also used by HTTP `app.upload`).

We **[Load the native plugin required by this operation](_load())**.
Return *[Offline helpers for upload validation and saving into # storage (also used by HTTP app.upload).](web_media_new(storage=`storage`))*.

## validate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| filename | any | required | Original or suggested filename |
| content_type | any | "application/octet-stream" | MIME type associated with the supplied content |
| size | any | required | Byte size checked by upload validation |
| max_bytes | any | 5242880 | Maximum accepted byte count; exceeding the limit must fail explicitly |
| types | any | None | Allowed MIME types for upload validation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Validate an upload filename, MIME type and byte size against explicit limits before storing anything. |

Use `filename`, `content_type`="application/octet-stream", `size`, `max_bytes`=5242880, `types`=None as the inputs for this operation.

Validate an upload filename, MIME type and byte size against explicit limits before storing anything.

Return *[Validate an upload filename, MIME type and byte size against explicit limits before storing anything.](web_upload_validate(filename=`filename`, content_type=`content_type`, size=`size`, max_bytes=`max_bytes`, types=`types`))*.

## save

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | required | Explicit filesystem path; honor the documented filesystem root and operation effects |
| key | any | None | Entry or field key within the selected resource |
| content_type | any | "application/octet-stream" | MIME type associated with the supplied content |
| prefix | any | "uploads/" | Object-key or generated-name prefix |
| storage | any | None | Storage resource created by the public storage constructor |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Save the selected local upload through the configured storage resource and return its storage result. |

Use `path`, `key`=None, `content_type`="application/octet-stream", `prefix`="uploads/", `storage`=None as the inputs for this operation.

We retain **st = storage**.
1. `st` == None
  We retain **`st` = [Read storage field](access(`self`, "storage"))**.
2. *

Return *[Save the selected local upload through the configured storage resource and return its storage result.](web_upload_save(storage=`st`, path=`path`, key=`key`, content_type=`content_type`, prefix=`prefix`))*.
