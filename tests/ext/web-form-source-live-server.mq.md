---
title: web form source live server
description: Request-context field sources — body only; author/slug stamped server-side.
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

`post_schema` =

| 字段 | 类型 | 必填 |
|------|------|------|
| id | integer | false |
| slug | text | true |
| title | text | true |
| content | text | true |

`comment_schema` =

| 字段 | 类型 | 必填 |
|------|------|------|
| id | integer | false |
| post_slug | text | true |
| author | text | true |
| body | text | true |
| created_at | text | false |

**store = > web.db url="sqlite:web-fixtures/data/form-source-live.db"**
> `store`.init name=posts fields=`post_schema`
> `store`.init name=comments fields=`comment_schema`
> `store`.exec sql="DELETE FROM comments"
> `store`.exec sql="DELETE FROM posts"

`seed` =

| slug | title | content |
|------|-------|---------|
| hello | Hello | Body of hello |

> `store`.insert table=posts rows=`seed`

`detail` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| body | posts.content | |

`post_q` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`comment_fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 | 来源 |
|------|------|------|------|------|------|
| body | Comment | textarea | true | | client |
| author | | text | true | | session.username |
| post_slug | | text | true | | route.slug |
| created_at | | text | false | | now |

`comment_rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| body | required | body required |
| author | required | login required |
| post_slug | required | slug required |

**cf = > web.form table="comments" action="insert"**
**cf = > `cf`.fields fields=`comment_fields`**
**cf = > `cf`.rules rules=`comment_rules`**

`comment_cards` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | comments.author | |
| body | comments.body | |
| meta | comments.created_at | |

`comment_q` =

| 字段 | 操作 | 值 |
|------|------|-----|
| post_slug | = | {slug} |

**post = > web.page title="Post" intro="<section class='post-comments'><h2>Comments</h2><div id='comment-list'></div><div id='comment-form-mount'></div></section>"**
**post = > `post`.compose_main main=`detail`**
**post = > `post`.query query=`post_q`**
**post = > `post`.detail detail=True**
**post = > `post`.compose_list main=`comment_cards` query=`comment_q` order="-created_at" target="#comment-list"**
**post = > `post`.compose_form form=`cf` id="comment" target="#comment-form-mount"**

`users` =

| 行 | 用户 | 密码 |
|----|------|------|
| 1 | alice | secret |

**app = > web.app page=`post` db=`store` port=18117**
**app = > `app`.route path="/post/{slug}" page=`post`**
**app = > `app`.auth users=`users` login_path="/login" login_redirect="/" session_ttl=3600**
**app = > `app`.mount_form id="comment" form=`cf`**

`api` =

| 路径 | 方法 | 表 | 条件 | 排序 | 上限 |
|------|------|----|------|------|------|
| /api/comments | GET | comments | | "-created_at" | 50 |

**app = > `app`.configure json=`api`**
> `app`.listen
