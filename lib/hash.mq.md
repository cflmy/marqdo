---
title: lib/hash
description: Digests and HMAC (Mid M3). Integrity / cache keys — not password hashing.
---

Digests and HMAC. For integrity and cache keys — not password storage.

## sha256
    + `text`

SHA-256 hex digest of text.

Caller: [hash.sha256] text=`s`

*> host_hash_sha256 text=`text`*

## sha1
    + `text`

SHA-1 hex digest of text.

*> host_hash_sha1 text=`text`*

## md5
    + `text`

MD5 hex digest of text.

*> host_hash_md5 text=`text`*

## hmac_sha256
    + `key`
    + `text`

HMAC-SHA256 of text with key.

*> host_hash_hmac_sha256 key=`key` text=`text`*
