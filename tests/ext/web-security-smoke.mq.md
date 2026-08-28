---
title: web security smoke (W3)
description: Password hash ABI + legacy plaintext login (offline).
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

<!-- load web plugin -->

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

<!-- argon2 hash must be non-empty PHC string -->

*hash = > web_password_hash password="secret"*
*h = hash[^hash]*
1. `h` != ""
  > print text=hash-ok
2. *
  > print text=hash-fail

<!-- legacy plaintext login still works for dev/gold -->

`users` =

| 行 | 用户 | 密码 |
|----|------|------|
| 1 | admin | secret |

*login = > web_auth_login username="admin" password="secret" users=`users` session_ttl=120*
*lok = login[^ok]*
1. `lok`
  > print text=auth-login-ok
2. *
  > print text=auth-login-fail

*bad = > web_auth_login username="admin" password="wrong" users=`users` session_ttl=120*
*bok = bad[^ok]*
1. `bok`
  > print text=auth-bad-should-fail
2. *
  > print text=auth-bad-reject-ok
