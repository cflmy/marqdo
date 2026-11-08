---
markup: occlusion
title: lib/hash
description: Digests and HMAC (Mid M3). Integrity / cache keys — not password hashing.
---

Digests and HMAC. For integrity and cache keys — not password storage.

Compute content digests and keyed message authentication.

Behavior and limits: MD5/SHA1 are compatibility checksums, not password storage. Use security auth for passwords and fs.file_hash for bounded-memory binary file hashing.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/hash.rs](../src/host/hash.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## sha256

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | SHA-256 hex digest of text |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/hash.mq.md sha256 'text="abc"' --effects none --json
```

SHA-256 hex digest of text.

Caller: [hash.sha256] text=`s`

Return *[SHA-256 hex digest of text](> host_hash_sha256 text=`text`)*.

## sha1

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | SHA-1 hex digest of text |

Use `text` as the inputs for this operation.

Effects: pure.

SHA-1 hex digest of text.

Return *[SHA-1 hex digest of text](> host_hash_sha1 text=`text`)*.

## md5

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | MD5 hex digest of text |

Use `text` as the inputs for this operation.

Effects: pure.

MD5 hex digest of text.

Return *[MD5 hex digest of text](> host_hash_md5 text=`text`)*.

## hmac_sha256

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | HMAC-SHA256 of text with key |

Use `key`, `text` as the inputs for this operation.

Effects: pure.

HMAC-SHA256 of text with key.

Return *[HMAC-SHA256 of text with key](> host_hash_hmac_sha256 key=`key` text=`text`)*.
