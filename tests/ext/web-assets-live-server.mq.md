---
title: web assets live server
description: Live favicon + head icon + images_html on home page.
import web:ext/web/web.mq.md
---

# main

`icons_table` =

|path|rel|type|url|
|---|---|---|---|
|"web-fixtures/public/favicon.png"|icon|"image/png"|"/favicon.ico"|

`head_table` =

|rel|href|type|
|---|---|---|
|apple-touch-icon|"/static/favicon.png"|"image/png"|

`images_table` =

|src|alt|class|loading|
|---|---|---|---|
|"/static/favicon.svg"|mark|brand|eager|

*home = > web.page title="Live assets" intro="<p>assets live</p>"*
*home = > `home`.head table=`head_table`*
*home = > `home`.images table=`images_table`*

*app = > web.app page=`home` host=127.0.0.1 port=18092*
*app = > `app`.static dir="web-fixtures/public" mount="/static"*
*app = > `app`.icons table=`icons_table`*

> print text=marqdo web assets live starting
> `app`.listen
