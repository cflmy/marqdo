---
markup: occlusion
title: lib/encoding
description: Base64, hex, and base32 encode/decode (Mid M1 + Mid2 M8). UTF-8 text round-trip.
---

Base64, hex, and base32 encode/decode. UTF-8 text round-trip.

Encode and decode UTF-8 text as Base64, hex or Base32.

Behavior and limits: These historical helpers round-trip text. Use bytes.from_base64 for arbitrary binary; decoding invalid UTF-8 as text is an error.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/encoding.rs](../src/host/encoding.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## base64_encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Base64-encode text |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/encoding.mq.md base64_encode 'text="Marqdo"' --effects none --json
```

Base64-encode text.

Caller: [encoding.base64_encode] text=`s`

Return *[Base64-encode text](> host_encoding_base64_encode text=`text`)*.

## base64_decode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Base64-decode text |

Use `text` as the inputs for this operation.

Effects: pure.

Base64-decode text.

Return *[Base64-decode text](> host_encoding_base64_decode text=`text`)*.

## hex_encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Hex-encode text |

Use `text` as the inputs for this operation.

Effects: pure.

Hex-encode text.

Return *[Hex-encode text](> host_encoding_hex_encode text=`text`)*.

## hex_decode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Hex-decode text |

Use `text` as the inputs for this operation.

Effects: pure.

Hex-decode text.

Return *[Hex-decode text](> host_encoding_hex_decode text=`text`)*.

## base32_encode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Base32-encode text (RFC 4648 alphabet; padding with =) |

Use `text` as the inputs for this operation.

Effects: pure.

Base32-encode text (RFC 4648 alphabet; padding with =).

Return *[Base32-encode text (RFC 4648 alphabet; padding with =)](> host_encoding_base32_encode text=`text`)*.

## base32_decode

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Base32-decode text |

Use `text` as the inputs for this operation.

Effects: pure.

Base32-decode text.

Return *[Base32-decode text](> host_encoding_base32_decode text=`text`)*.
