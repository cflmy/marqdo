---
markup: occlusion
title: ext/security/auth
description: Auth Resource (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

Read [the security entry](index.mq.md) to distinguish credential/session operations, role checks and OIDC integration. This leaf owns the native authentication resource; endpoint metadata still controls route authorization. Prepare the native web plugin, keep secrets out of examples and logs, and verify both denied and accepted credentials before changing session behavior.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/security: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

# auth

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| users | any | required | Configured user store used by authentication |
| session_ttl | any | 3600 | Session lifetime in seconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Session/auth helper. Constructs a config object; `login` validates against the users table. To gate `/admin` on this app, use `app.auth users=…` instead. |

Use `users`, `session_ttl`=3600 as the inputs for this operation.

Session/auth helper. Constructs a config object; `login` validates against the users table. To gate `/admin` on this app, use `app.auth users=…` instead.

We **[Load the native plugin required by this operation](_load())**.
Return *[Session/auth helper. Constructs a config object; login validates against the users table. To gate /admin on this app, use app.auth users=… instead.](web_auth_new(users=`users`, session_ttl=`session_ttl`))*.

## login

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| username | any | required | User name in the configured user store |
| password | any | required | Credential supplied for verification or hashing; do not log it |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Validate credentials against the users table and create a session. Returns `{ok, session_id, username, role}`. |

Use `username`, `password` as the inputs for this operation.

Validate credentials against the users table and create a session. Returns `{ok, session_id, username, role}`.

We retain **`users` = [Read users field](access(`self`, "users"))**.
We retain **`ttl` = [Read session_ttl field](access(`self`, "session_ttl"))**.
Return *[Validate credentials against the users table and create a session. Returns {ok, session_id, username, role}.](web_auth_login(username=`username`, password=`password`, users=`users`, session_ttl=`ttl`))*.

## check

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| session_id | any | required | Session identifier returned by login |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Returns `{ok, username, role}` when the session is valid. |

Use `session_id` as the inputs for this operation.

Returns `{ok, username, role}` when the session is valid.

Return *[Returns {ok, username, role} when the session is valid.](web_auth_check(session_id=`session_id`))*.

## logout

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| session_id | any | required | Session identifier returned by login |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Destroy the session. |

Use `session_id` as the inputs for this operation.

Destroy the session.

Return *[Destroy the session.](web_auth_logout(session_id=`session_id`))*.

## hash_password

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| password | any | required | Credential supplied for verification or hashing; do not log it |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Hash a plaintext password for storage in admin user tables (argon2id). Store the returned `hash` in the `password` column; login verifies automatically. |

Use `password` as the inputs for this operation.

Hash a plaintext password for storage in admin user tables (argon2id). Store the returned `hash` in the `password` column; login verifies automatically.

Return *[Hash a plaintext password for storage in admin user tables (argon2id). Store the returned hash in the password column; login verifies automatically.](web_password_hash(password=`password`))*.
