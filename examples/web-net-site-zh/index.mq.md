---
title: 网络扩展站（中文）
description: 首页 + /about + /new（表单）+ /tools（lib/net）+ /admin（登录鉴权）+ /live（WebSocket）。
导入 网页:ext/web/网页.mq.md
import net:lib/网络.mq.md
import text:lib/text.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

`首页` =

| 组件 | 样式 |
|------|------|
| nav.`导航` | shell.`顶栏` |
| side.`侧栏` | shell.`侧栏面板` |
| foot.`页脚` | |

`列表` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | articles.`articles`.title | shell.`卡片标题` |
| body | articles.`articles`.body | shell.`卡片正文` |

`文章字段` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | 标题 | text | true | |
| body | 正文 | textarea | false | |

`文章规则` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| title | max:120 | 标题太长 |
| body | max:8000 | 正文太长 |

*store = > db.打开*
*page = > 网页.页面 标题="网络扩展站" 引言="<h1>网络扩展站</h1><p>SQLite 文章列表、WebSocket 实时小部件、登录门禁后台、以及标准库 lib/net 的解析工具。</p><h2 id=\"live\">实时 WebSocket</h2><p id=\"live-status\">connecting…</p><input id=\"live-msg\" placeholder=\"输入一条消息…\"><button id=\"live-send\" disabled>发送</button><div id=\"live-out\"></div><script src=\"/static/live.js\"></script>"*
*page = > page.组件装配 组件=`首页`*
*page = > page.主体装配 主体=`列表`*

*about = > 网页.页面 标题="关于" 引言="<h1>关于</h1><p>演示网络扩展后的 web 库：会话、登录鉴权、WebSocket、multipart/cookie 解析。</p>"*
*about = > about.组件装配 组件=`首页`*

<!-- 工具页：用 lib/net 解析展示 -->

*解析请求 = > net.解析Cookie 内容="session=abc123; theme=dark"*
*请求块 = 解析请求[^1]*

*解析响应 = > net.解析Cookie 内容="id=42; Path=/; HttpOnly; SameSite=Lax" 是响应头=True*
*响应块 = 解析响应[^1]*

`边界` = ----b

`正文` = "--`边界`\nContent-Disposition: form-data; name=\"title\"\n\n你好\n--`边界`\nContent-Disposition: form-data; name=\"封面\"; filename=\"pic.png\"\nContent-Type: image/png\n\n<字节>\n--`边界`--\n"

*多部分 = > net.解析多部分 正文=`正文` 边界=`边界`*
*字段块 = 多部分[^1]*
*文件块 = 多部分[^2]*

`工具段` =

| 段 |
|----|
| <h1>网络工具</h1><p>由 lib/net 提供的解析演示：解析Cookie 与 解析多部分。</p><section class="content cards"> |
| <article><h2>Cookie 请求头</h2><p> |
| `请求块`[^name]=`请求块`[^value] |
| </p></article><article><h2>Set-Cookie 响应头</h2><p> |
| `响应块`[^name]=`响应块`[^value] · HttpOnly=`响应块`[^http_only] |
| </p></article><article><h2>多部分字段</h2><p> |
| `字段块`[^name]=`字段块`[^value] |
| </p></article><article><h2>多部分文件</h2><p> |
| `文件块`[^name] · `文件块`[^filename] · `文件块`[^content_type] |
| </p></article></section> |

*工具引言 = > text.str_join xs=`工具段` sep=""*
*tools = > 网页.页面 标题="工具" 引言=`工具引言`*
*tools = > tools.组件装配 组件=`首页`*

*article_form = > 网页.表单 表="articles" 动作="插入"*
*article_form = > article_form.字段 字段=`文章字段`*
*article_form = > article_form.规则 规则=`文章规则`*

*new = > 网页.页面 标题="发布" 引言="<h1>发布</h1><p>表单插入到 articles 表。</p>"*
*new = > new.组件装配 组件=`首页`*
*new = > new.表单装配 id="article" 表单=article_form*

`管理员` =

| 行 | 用户名 | 密码 |
|----|--------|------|
| 1 | admin | secret |

*app = > 网页.应用 页面=page 数据库=store 后台=True 主机="127.0.0.1" 端口=18084*
*app = > app.路由 路径="/about" 页面=about*
*app = > app.路由 路径="/new" 页面=new*
*app = > app.路由 路径="/tools" 页面=tools*
*app = > app.静态 目录="public" 挂载="/static"*
*app = > app.路由实时 路径="/live" 回显=True*
*app = > app.鉴权 用户表=`管理员` 会话时长=3600*
> `app`.监听