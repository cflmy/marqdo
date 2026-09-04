---
title: web C1 gate match / on_deny / exclude
description: Segment-boundary prefix; auth default redirect; custom gate fields.
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

*首页 = > web.page title="home"*

`用户表` =
| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | secret | admin |

*应用 = > web.app page=`首页` admin=True*
*应用 = > `应用`.auth users=`用户表`*
*gates = > json.get value=`应用` key="gates"*
*g0 = > at value=`gates` index=0*
*path = > json.get value=`g0` key="path"*
*match = > json.get value=`g0` key="match"*
*deny = > json.get value=`g0` key="on_deny"*
*login = > json.get value=`应用` key="login_path"*
1. `path` == "/admin"
  1. `match` == "prefix"
    1. `deny` == "redirect"
      1. `login` == "/admin/login"
        > print text=auth-default-gate-ok
      2. *
        > print text=login-path-fail
        > sys.exit code=1
    2. *
      > print text=on-deny-fail
      > sys.exit code=1
  2. *
    > print text=match-fail
    > sys.exit code=1
2. *
  > print text=path-fail
  > sys.exit code=1

*应用2 = > web.app page=`首页` admin=False*
*应用2 = > `应用2`.gate path="/desk" roles="admin" match="prefix" on_deny="redirect" exclude="/desk/login"*
*gates2 = > json.get value=`应用2` key="gates"*
*g1 = > at value=`gates2` index=0*
*p2 = > json.get value=`g1` key="path"*
*m2 = > json.get value=`g1` key="match"*
*d2 = > json.get value=`g1` key="on_deny"*
*ex = > json.get value=`g1` key="exclude"*
1. `p2` == "/desk"
  1. `m2` == "prefix"
    1. `d2` == "redirect"
      1. `ex`
        > print text=custom-gate-ok
      2. *
        > print text=exclude-fail
        > sys.exit code=1
    2. *
      > print text=custom-deny-fail
      > sys.exit code=1
  2. *
    > print text=custom-match-fail
    > sys.exit code=1
2. *
  > print text=custom-path-fail
  > sys.exit code=1

*应用3 = > web.app page=`首页` admin=False*
*应用3 = > `应用3`.gate path="/write*" roles="admin,author"*
*gates3 = > json.get value=`应用3` key="gates"*
*g3 = > at value=`gates3` index=0*
*p3 = > json.get value=`g3` key="path"*
1. `p3`
  > print text=star-gate-ok
2. *
  > print text=star-gate-fail
  > sys.exit code=1

> print text=c1-gate-ok
