---
title: web C3 style strict + head defer/version
description: Offline markers for quoted CSS cells and head script lifecycle.
import web:ext/web/web.mq.md
import sys:lib/sys.mq.md
---

# main

`好` =
| 选择器 | 属性 | 值 |
|--------|------|-----|
| .box | grid-column | "1 / 5" |

*css = > web.make_style name="t" table=`好` strict=True*
*a = > split value=`css` sep="1 / 5"*
*na = > len value=`a`*
1. `na` > 1
  > print text=quoted-css-ok
2. *
  > print text=quoted-css-missing
  > sys.exit code=1

`头` =
| 关系 | 地址 | 推迟 | 版本 |
|------|------|------|------|
| script | /static/a.js | 真 | 3 |
| stylesheet | /static/t.css | | 1 |

*页 = > web.page title="c3" asset_version="appv"*
*页 = > `页`.head table=`头`*
*html = > `页`.render*
*b = > split value=`html` sep="a.js?v=3"*
*nb = > len value=`b`*
*c = > split value=`html` sep="t.css?v=1"*
*nc = > len value=`c`*
*d0 = > split value=`html` sep="script defer"*
*nd0 = > len value=`d0`*
1. `nb` > 1
  1. `nc` > 1
    1. `nd0` > 1
      > print text=defer-version-ok
    2. *
      > print text=defer-attr-missing
      > sys.exit code=1
  2. *
    > print text=css-version-missing
    > sys.exit code=1
2. *
  > print text=defer-script-missing
  > sys.exit code=1

`头2` =
| 关系 | 地址 | 推迟 |
|------|------|------|
| script | /static/b.js | 真 |

*页2 = > web.page title="c3b" asset_version="2026-09-04"*
*页2 = > `页2`.head table=`头2`*
*h2 = > `页2`.render*
*d = > split value=`h2` sep="b.js?v=2026-09-04"*
*nd = > len value=`d`*
1. `nd` > 1
  > print text=asset-version-ok
2. *
  > print text=asset-version-missing
  > sys.exit code=1

`头3` =
| 关系 | 地址 |
|------|------|
| script | /static/sync.js |

*页3 = > web.page title="c3c"*
*页3 = > `页3`.head table=`头3`*
*h3 = > `页3`.render*
*e = > split value=`h3` sep="sync.js"*
*ne = > len value=`e`*
*f = > split value=`h3` sep="script defer"*
*nf = > len value=`f`*
1. `ne` > 1
  1. `nf` == 1
    > print text=sync-compat-ok
  2. *
    > print text=unexpected-defer
    > sys.exit code=1
2. *
  > print text=sync-script-missing
  > sys.exit code=1

> print text=c3-style-head-ok
