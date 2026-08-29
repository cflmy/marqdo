---
title: web assets smoke
description: Offline icons / head / images assembly (no listen).
import web:ext/web/web.mq.md
import json:lib/json.mq.md
---

# main

`icons_table` =

|path|rel|type|sizes|url|
|---|---|---|---|---|
|"web-fixtures/public/favicon.png"|icon|"image/png"|32x32|"/favicon.ico"|
|"web-fixtures/public/favicon.svg"|icon|"image/svg+xml"|any|"/static/favicon.svg"|

`head_table` =

|rel|href|type|media|
|---|---|---|---|
|icon|"/favicon.ico"|"image/png"|
|stylesheet|"/static/print.css"|"text/css"|print|
|script|"/static/live.js"||

`images_table` =

|src|alt|class|href|width|loading|caption|
|---|---|---|---|---|---|---|
|"/static/logo.svg"|Marqdo|brand-logo|"/"|48|eager|Brand|

`seo` =

| 行 | key | value |
|----|-----|-------|
| 1 | description | asset smoke |
| 2 | icon | "/favicon.ico" |
| 3 | og:image | "/static/logo.svg" |

*home = > web.page title="Assets"*
*home = > `home`.head table=`head_table`*
*home = > `home`.images table=`images_table`*
*home = > `home`.meta meta=`seo`*

*ih = > json.get value=`home` key="images_html"*
1. `ih` != ""
  > print text=images-ok
2. *
  > print text=images-fail

*hd = > json.get value=`home` key="head"*
*n = > len `hd`*
1. `n` >= 3
  > print text=head-ok
2. *
  > print text=head-fail

*meta = > json.get value=`home` key="meta"*
*icon = > json.get value=`meta` key="icon"*
1. `icon` == "/favicon.ico"
  > print text=meta-icon-ok
2. *
  > print text=meta-icon-fail

*app = > web.app page=`home` host=127.0.0.1 port=18091*
*app = > `app`.static dir="web-fixtures/public" mount="/static"*
*app = > `app`.icons table=`icons_table`*

*icons = > json.get value=`app` key="icons"*
*ni = > len `icons`*
1. `ni` == 2
  > print text=icons-ok
2. *
  > print text=icons-fail

*sh = > json.get value=`app` key="site_head"*
*ns = > len `sh`*
1. `ns` >= 1
  > print text=site-head-ok
2. *
  > print text=site-head-fail

*preview = > web.make_images table=`images_table`*
1. `preview` != ""
  > print text=make-images-ok
2. *
  > print text=make-images-fail

*hprev = > web.make_head table=`head_table`*
1. `hprev` != ""
  > print text=make-head-ok
2. *
  > print text=make-head-fail
