---
title: lib/hash
description: Digests and HMAC (Mid M3). Integrity / cache keys — not password hashing.
---

## sha256
    + `text`

**> host_hash_sha256 text=`text`**

## sha1
    + `text`

**> host_hash_sha1 text=`text`**

## md5
    + `text`

**> host_hash_md5 text=`text`**

## hmac_sha256
    + `key`
    + `text`

**> host_hash_hmac_sha256 key=`key` text=`text`**
