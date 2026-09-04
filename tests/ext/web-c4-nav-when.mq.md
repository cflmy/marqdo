---
title: web C4 conditional nav when/media
description: Offline render markers for nav when= and media= columns.
import web:ext/web/web.mq.md
import table:lib/table.mq.md
import sys:lib/sys.mq.md
---

# main

`链` =
| 属性 | 值 | 样式 | 媒体 | 当 |
|------|-----|------|------|-----|
| Home | / | | | |
| Desk | /desk | | (max-width: 720px) | |
| Admin | /admin | | | auth |
| Login | /login | | | guest |
| Gone | /gone | | | hide |

*客 = > web.page title="c4-guest" shell_css="off"*
*`客` = > table.put in=`客` at="nav" value=`链`*
*`客` = > table.put in=`客` at="_logged_in" value=False*
*hg = > `客`.render*

*a = > split value=`hg` sep="/login"*
*na = > len value=`a`*
*b = > split value=`hg` sep="/admin"*
*nb = > len value=`b`*
*c = > split value=`hg` sep="/gone"*
*nc = > len value=`c`*
*d = > split value=`hg` sep="nav-mq-0"*
*nd = > len value=`d`*
*e = > split value=`hg` sep="@media not (max-width: 720px)"*
*ne = > len value=`e`*
1. `na` > 1
  1. `nb` == 1
    1. `nc` == 1
      1. `nd` > 1
        1. `ne` > 1
          > print text=guest-media-ok
        2. *
          > print text=media-css-missing
          > sys.exit code=1
      2. *
        > print text=media-class-missing
        > sys.exit code=1
    2. *
      > print text=hide-leaked
      > sys.exit code=1
  2. *
    > print text=auth-leaked-for-guest
    > sys.exit code=1
2. *
  > print text=guest-login-missing
  > sys.exit code=1

*户 = > web.page title="c4-user" shell_css="off"*
*`户` = > table.put in=`户` at="nav" value=`链`*
*`户` = > table.put in=`户` at="_logged_in" value=True*
*hu = > `户`.render*
*f = > split value=`hu` sep="/admin"*
*nf = > len value=`f`*
*g = > split value=`hu` sep="/login"*
*ng = > len value=`g`*
1. `nf` > 1
  1. `ng` == 1
    > print text=auth-ok
  2. *
    > print text=guest-leaked-for-user
    > sys.exit code=1
2. *
  > print text=auth-admin-missing
  > sys.exit code=1

> print text=c4-nav-ok
