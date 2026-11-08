---
markup: occlusion
---

Recoverable outcomes use one ordinary record: ok/value/error. A failure's error
contains code/message/operation. Inspect ok before consuming value. Host policy
checks still apply; a Result wrapper never authorizes an effect.

Compose recoverable outcomes with one data contract.

Behavior and limits: Check ok before value; HTTP status and child exit code remain domain-level results; errors have code/message/operation. Use try_ operations to recover known IO/parse failures. Result construction does not catch arbitrary runtime errors or grant permission.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/result.rs](../src/host/result.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## ok

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Successful operation value, including None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Successful Result |

Use `value` as the inputs for this operation.

Delegate to ok with the supplied inputs, preserving its result and failure behavior.

Effects: pure.

`out` =

| ok | value | error |
|----|-------|-------|
| True | `value` | None |

Return *out*.

## error

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | text | required | Stable machine-readable category |
| message | text | required | Human-readable explanation |
| operation | text | "application" | Operation that failed |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Failed Result |

Use `code`, `message`, `operation`="application" as the inputs for this operation.

Delegate to error with the supplied inputs, preserving its result and failure behavior.

Effects: pure.

`failure` =

| code | message | operation |
|------|---------|-----------|
| `code` | `message` | `operation` |

`out` =

| ok | value | error |
|----|-------|-------|
| False | None | `failure` |

Return *out*.

## value_or

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| result | map | required | Result from a recoverable operation |
| default | any | None | Explicit fallback on failure |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Successful value or fallback |

Use `result`, `default`=None as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/result.mq.md value_or 'result={"ok":false,"value":null,"error":{"code":"missing","message":"absent","operation":"example"}}' 'default="fallback"' --effects none --json
```

Read value only when ok is true, including a successful None. Use the fallback only on failure; zero and empty text remain valid values.

1. [operation succeeded](access(`result`, "ok"))
  Return *[successful value](access(result, "value"))*.
2. *
  Return *default*.
