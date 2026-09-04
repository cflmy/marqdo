---
title: web C2 shell_css + layout
description: Offline render markers for shell modes and layouts.
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

`链` =
| 标签 | 链接 |
|------|------|
| Home | / |
| Desk | /desk |

*页 = > web.page title="c2" shell_css="off" layout="stacked"*
*页 = > json.set map=`页` key="nav" value=`链`*
*页 = > json.set map=`页` key="sidebar" value=`链`*
*html = > `页`.render*

*a = > split value=`html` sep="layout-stacked"*
*na = > len value=`a`*
*b = > split value=`html` sep="has-sidebar"*
*nb = > len value=`b`*
*c = > split value=`html` sep="grid-template-columns:14rem"*
*nc = > len value=`c`*
1. `na` > 1
  1. `nb` == 1
    1. `nc` == 1
      > print text=stacked-off-ok
    2. *
      > print text=grid-still-present
      > sys.exit code=1
  2. *
    > print text=has-sidebar-leak
    > sys.exit code=1
2. *
  > print text=stacked-missing
  > sys.exit code=1

*裸 = > web.page title="bare" layout="bare" shell_css="minimal"*
*裸 = > json.set map=`裸` key="nav" value=`链`*
*裸 = > json.set map=`裸` key="sidebar" value=`链`*
*h2 = > `裸`.render*
*d = > split value=`h2` sep="<aside"*
*nd = > len value=`d`*
*e = > split value=`h2` sep="--ink:"*
*ne = > len value=`e`*
1. `nd` == 1
  1. `ne` > 1
    > print text=bare-minimal-ok
  2. *
    > print text=minimal-vars-missing
    > sys.exit code=1
2. *
  > print text=bare-aside-leak
  > sys.exit code=1

> print text=c2-shell-ok
