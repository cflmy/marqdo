---
title: web network-extension smoke
description: Session/auth/ws ABI primitives (offline).
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

<!-- load web plugin so ABI primitives are registered -->

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

<!-- admin users table (row-oriented records) -->

`users` =

| 行 | 用户 | 密码 |
|----|------|------|
| 1 | admin | secret |
| 2 | 站长 | pw123 |

<!-- session CRUD -->

**sess = > web_session_new ttl_sec=120**
**sid = [id](sess)**
1. `sid` != ""
  > print text=session-new-ok
2. *
  > print text=session-new-fail

**set1 = > web_session_set id=`sid` key="theme" value="dark"**
**ok1 = [ok](set1)**
1. `ok1`
  > print text=session-set-ok
2. *
  > print text=session-set-fail

**get1 = > web_session_get id=`sid` key="theme"**
**v1 = [value](get1)**
1. `v1` == "dark"
  > print text=session-get-ok
2. *
  > print text=session-get-fail

**del1 = > web_session_del id=`sid` key="theme"**
**ok2 = [ok](del1)**
1. `ok2`
  > print text=session-del-ok
2. *
  > print text=session-del-fail

**get2 = > web_session_get id=`sid` key="theme"**
**ok3 = [ok](get2)**
1. `ok3`
  > print text=session-del-confirm-fail
2. *
  > print text=session-del-confirm

<!-- auth login/check/logout -->

**login = > web_auth_login username="admin" password="secret" users=`users` session_ttl=120**
**lok = [ok](login)**
1. `lok`
  > print text=auth-login-ok
2. *
  > print text=auth-login-fail

**sid2 = [session_id](login)**
**check = > web_auth_check session_id=`sid2`**
**cok = [ok](check)**
1. `cok`
  > print text=auth-check-ok
2. *
  > print text=auth-check-fail

**badlogin = > web_auth_login username="admin" password="wrong" users=`users` session_ttl=120**
**bok = [ok](badlogin)**
1. `bok`
  > print text=auth-bad-should-fail
2. *
  > print text=auth-bad-ok

**zhlogin = > web_auth_login username="站长" password="pw123" users=`users` session_ttl=120**
**zhok = [ok](zhlogin)**
1. `zhok`
  > print text=auth-zh-ok
2. *
  > print text=auth-zh-fail

**logout = > web_auth_logout session_id=`sid2`**
**dok = [ok](logout)**
1. `dok`
  > print text=auth-logout-ok
2. *
  > print text=auth-logout-fail

**check2 = > web_auth_check session_id=`sid2`**
**cok2 = [ok](check2)**
1. `cok2`
  > print text=auth-logout-confirm-fail
2. *
  > print text=auth-logout-confirm

<!-- route ws + app auth (offline) -->

**pg = > web.page title="ws-test"**
**app = > web.app page=`pg` admin=True**
**app = > `app`.route_ws path="/live" echo=True**
**routes = [ws_routes](app)**
**has_live = [/live](routes)**
1. `has_live`
  > print text=ws-route-ok
2. *
  > print text=ws-route-fail

**app = > `app`.auth users=`users` session_ttl=120**
**authobj = [auth](app)**
**ausers = [users](authobj)**
**au1 = [用户]([1](ausers))**
**ou1 = [用户]([1](users))**
1. `au1` == `ou1`
  > print text=app-auth-ok
2. *
  > print text=app-auth-fail

<!-- via ext/web author surface -->

**authz = > web.auth users=`users` session_ttl=120**
**login2 = > `authz`.login username="admin" password="secret"**
**lok2 = [ok](login2)**
1. `lok2`
  > print text=ext-auth-ok
2. *
  > print text=ext-auth-fail

**ws = > web.ws timeout_sec=1**
**badurl = > `ws`.connect url="ws://127.0.0.1:1" message="hi"**
**bwok = [ok](badurl)**
1. `bwok`
  > print text=ws-connect-should-fail
2. *
  > print text=ws-connect-error-ok