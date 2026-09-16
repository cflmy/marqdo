---
title: web compose_intro bind table
description: Intro uses the same front/back/style bind as compose_main.
import web:ext/web/web.mq.md
import sys:lib/sys.mq.md
---

# main

`intro` =

| front | back | style |
|-------|------|-------|
| kicker | Demo kicker | kicker |
| title | Hello | intro_title |
| lede | "Try code and [link](/x)." | lede |
| claim | A | claim |
| claim | B | claim |
| step | First step | step |

**page = > web.page title="Intro bind"**
**page = > page.compose_intro intro=intro**
**html = > page.render**

**a = > split value=`html` sep="class=\"kicker\""**
**na = > len value=`a`**
**b = > split value=`html` sep="class=\"intro_title\""**
**nb = > len value=`b`**
**c = > split value=`html` sep="href=\"/x\""**
**nc = > len value=`c`**
**d = > split value=`html` sep="class=\"claim\""**
**nd = > len value=`d`**
**e = > split value=`html` sep="class=\"steps\""**
**ne = > len value=`e`**
1. `na` > 1
  1. `nb` > 1
    1. `nc` > 1
      1. `nd` > 1
        1. `ne` > 1
          > print text=compose-intro-ok
        2. *
          > print text=compose-intro-steps-missing
          > sys.exit code=1
      2. *
        > print text=compose-intro-claim-missing
        > sys.exit code=1
    2. *
      > print text=compose-intro-link-missing
      > sys.exit code=1
  2. *
    > print text=compose-intro-title-missing
    > sys.exit code=1
2. *
  > print text=compose-intro-kicker-missing
  > sys.exit code=1
