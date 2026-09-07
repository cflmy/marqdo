---
title: lib/secrets
description: Cryptographic tokens (Mid M3). Do not use math.random for secrets.
---

## token_hex
    + `n`=None

Byte length `n` (default 16). Returns lowercase hex of length `2*n`.

**> host_secrets_token_hex n=`n`**

## token_urlsafe
    + `n`=None

Byte length `n` (default 16). URL-safe Base64 without padding.

**> host_secrets_token_urlsafe n=`n`**
