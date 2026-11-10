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

## attach

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared web application |
| users | any | required | Credential table or native database-backed user configuration |
| session_ttl | number | 3600 | Session lifetime in seconds |
| admin_prefix | text | "/admin" | Native administration prefix |
| login_path | text | "/login" | Declared login route |
| login_redirect | text | "" | Optional successful login destination |
| logout_redirect | text | "" | Optional logout destination |
| register | bool | False | Explicitly enable user registration |
| register_path | text | "/register" | Registration route when enabled |
| default_role | text | "user" | Role assigned to a registered user |
| session_url | text | "" | Optional session backend URL |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Updated app with native login/session policy |

Use `app`, `users`, `session_ttl`=3600, `admin_prefix`="/admin", `login_path`="/login", `login_redirect`="", `logout_redirect`="", `register`=False, `register_path`="/register", `default_role`="user" and `session_url`="" before web.listen. This resource owns the login/session policy, while Endpoint auth metadata and explicit gates decide which requests need a session or role. Retain the returned app; construction does not authenticate a request or start a server. Credential tables in local examples are fixtures; real user stores should contain the hashes produced by hash_password. Registration is off until deliberately enabled. Session lifetime is in seconds, and external session backends require their own connection configuration.

We **[Load the native authentication implementation](plugin.load(path=plugin.native_path(name="web")))**.
Return *[Bind the selected login and session policy](web_app_auth(app=`app`, users=`users`, session_ttl=`session_ttl`, admin_prefix=`admin_prefix`, login_redirect=`login_redirect`, logout_redirect=`logout_redirect`, login_path=`login_path`, register=`register`, register_path=`register_path`, default_role=`default_role`, session_url=`session_url`))*.


## gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared application retaining its actual resources |
| path | text | required | Explicit URL path to protect |
| roles | any | "admin" | Allowed role CSV or list |
| permissions | text | "" | Optional permission CSV; native any-match policy |
| match | text | "prefix" | prefix or exact path matching |
| on_deny | text | "forbid" | forbid or redirect anonymous requests to login |
| exclude | any | None | Explicit path exclusions; native login/logout exclusions remain |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Updated app with a server-enforced gate; no listener starts |

Use `app`, `path`, `roles`="admin", `permissions`="", `match`="prefix", `on_deny`="forbid" and `exclude`=None before listening, with plugin permission. Retain the returned app. This security-owned operation protects real requests; hiding a navigation link does not do that. It reuses the existing native gate and does not mutate Document content or invent routes.

Prefix matching includes the selected path and its slash descendants. The runtime keeps login and logout available to avoid authentication loops. When permissions are supplied, native permission matching applies instead of the role list; any required permission may match. Redirect applies to anonymous denial, while a signed-in subject without access receives forbidden. Test both direct denied requests and authenticated access, and gate the actual form/API endpoint separately if its data is private.

We **[Load the native access-policy implementation](_load())**.
Return *[Bind the explicit server access rule](web_app_gate(app=`app`, path=`path`, roles=`roles`, permissions=`permissions`, match=`match`, on_deny=`on_deny`, exclude=`exclude`))*.


# auth

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| users | any | required | Configured user store used by authentication |
| session_ttl | any | 3600 | Session lifetime in seconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Session/auth helper. Constructs a config object; `login` validates against the users table. To bind native login and protect `/admin`, call attach and gate before listening. |

Use `users`, `session_ttl`=3600 as the inputs for this operation.

Session/auth helper. Constructs a config object; `login` validates against the users table. To bind native login and protect `/admin`, call attach and gate before listening.

We **[Load the native plugin required by this operation](_load())**.
Return *[Session/auth helper. Constructs a config object; login validates against the users table. To bind server login and gates, use this module's attach function before web.listen.](web_auth_new(users=`users`, session_ttl=`session_ttl`))*.

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
