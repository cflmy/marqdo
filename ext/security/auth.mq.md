---
title: ext/security/auth
description: Auth Resource (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/security: native web plugin not found
  > sys.exit code=1
****

# auth
    + `users`
    + `session_ttl`=3600

Session/auth helper. Constructs a config object; `login` validates against the users table. To gate `/admin` on this app, use `app.auth users=…` instead.

> _load
*> web_auth_new users=`users` session_ttl=`session_ttl`*

## login
    + `username`
    + `password`

Validate credentials against the users table and create a session. Returns `{ok, session_id, username, role}`.

**users = [users](self)**
**ttl = [session_ttl](self)**
*> web_auth_login username=`username` password=`password` users=`users` session_ttl=`ttl`*

## check
    + `session_id`

Returns `{ok, username, role}` when the session is valid.

*> web_auth_check session_id=`session_id`*

## logout
    + `session_id`

Destroy the session.

*> web_auth_logout session_id=`session_id`*

## hash_password
    + `password`

Hash a plaintext password for storage in admin user tables (argon2id). Store the returned `hash` in the `password` column; login verifies automatically.

*> web_password_hash password=`password`*

