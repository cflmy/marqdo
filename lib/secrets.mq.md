---
markup: occlusion
title: lib/secrets
description: Cryptographic tokens (Mid M3). Do not use math.random for secrets.
---

Cryptographic tokens. Do not use math.random for secrets.

Generate cryptographically random tokens.

Behavior and limits: n counts input bytes. Do not use math.random for credentials. Browser entropy depends on the runtime bridge; do not silently substitute a pseudo-random generator.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/secrets.rs](../src/host/secrets.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## token_hex

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| n | any | None | Requested byte count; None uses the helper default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Byte length n (default 16) |

Use `n`=None as the inputs for this operation.

Effects: random.read.

Call this function directly:

```sh
marqdo call lib/secrets.mq.md token_hex n=16 --effects random.read --json
```

Byte length n (default 16). Returns lowercase hex of length 2*n.

Caller: [secrets.token_hex] n=16

Return *[Byte length n (default 16)](> host_secrets_token_hex n=`n`)*.

## token_urlsafe

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| n | any | None | Requested byte count; None uses the helper default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Byte length n (default 16) |

Use `n`=None as the inputs for this operation.

Effects: random.read.

Byte length n (default 16). URL-safe Base64 without padding.

Return *[Byte length n (default 16)](> host_secrets_token_urlsafe n=`n`)*.
