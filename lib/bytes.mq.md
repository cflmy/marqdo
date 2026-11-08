---
markup: occlusion
title: bytes
---

Bytes are ordinary records with `_type: bytes` and `base64`, portable across CLI, JSON, plugins and sessions; no new language type.

Carry arbitrary binary data through ordinary portable records.

Behavior and limits: A bytes record contains _type=bytes and canonical Base64. Byte offsets are zero-based; UTF-8 decoding is explicit and strict. read_chunk has no long-lived file handle.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/fs_ops.rs](../src/host/fs_ops.rs)、[src/host/encoding.rs](../src/host/encoding.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## from_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Encode text as UTF-8 bytes |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/bytes.mq.md from_text 'text="Marqdo"' --effects none --json
```

Encode text as UTF-8 bytes.

Return *[Encode text as UTF-8 bytes](> host_bytes_from_text text=`text`)*.

## from_base64

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Decode Base64 as arbitrary bytes; no UTF-8 assumption |

Use `text` as the inputs for this operation.

Effects: pure.

Decode Base64 as arbitrary bytes; no UTF-8 assumption.

Return *[Decode Base64 as arbitrary bytes; no UTF-8 assumption](> host_bytes_from_base64 text=`text`)*.

## to_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Portable bytes record containing _type=bytes and base64 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Decode UTF-8 strictly; invalid byte sequences fail |

Use `data` as the inputs for this operation.

Effects: pure.

Decode UTF-8 strictly; invalid byte sequences fail.

Return *[Decode UTF-8 strictly; invalid byte sequences fail](> host_bytes_to_text data=`data`)*.

## to_base64

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Portable bytes record containing _type=bytes and base64 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Return canonical padded Base64 |

Use `data` as the inputs for this operation.

Effects: pure.

Return canonical padded Base64.

Return *[Return canonical padded Base64](> host_bytes_to_base64 data=`data`)*.

## len

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Portable bytes record containing _type=bytes and base64 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Count bytes, not characters |

Use `data` as the inputs for this operation.

Effects: pure.

Count bytes, not characters.

Return *[Count bytes, not characters](> host_bytes_len data=`data`)*.

## slice

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Portable bytes record containing _type=bytes and base64 |
| start | number | required | Inclusive zero-based slice start |
| end | number | required | Exclusive zero-based slice end; None selects through the end |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Return a zero-based half-open byte slice; endpoints clamp to length |

Use `data`, `start`, `end` as the inputs for this operation.

Effects: pure.

Return a zero-based half-open byte slice; endpoints clamp to length.

Return *[Return a zero-based half-open byte slice; endpoints clamp to length](> host_bytes_slice data=`data` start=`start` end=`end`)*.
