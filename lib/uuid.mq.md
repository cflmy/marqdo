---
markup: occlusion
title: lib/uuid
description: UUID v4 (Mid M6). Cryptographic random via getrandom.
---

Random UUID helpers. Prefer this over rolling your own ids.

Create UUID v4 identifiers.

Behavior and limits: Identifiers use system entropy and are not reproducible constants. Do not treat UUID strings as authorization tokens.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/uuid_ops.rs](../src/host/uuid_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## v4

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | New UUID version 4 string |

Effects: random.read.

Call this function directly:

```sh
marqdo call lib/uuid.mq.md v4  --effects random.read --json
```

New UUID version 4 string.

Caller: [uuid.v4]

Return *[New UUID version 4 string](host_uuid_v4())*.
