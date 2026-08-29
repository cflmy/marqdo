---
title: Marqdo 博客
description: 用 web 扩展库搭建的博客系统示例：首页列表、动态文章详情、标签归档、登录门禁后台。
导入 网页:ext/web/网页.mq.md
import theme:styles/theme.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import db:db/index.mq.md
---

# main

`首页` =

| 组件 | 样式 |
|------|------|
| nav.`导航` | |
| side.`侧栏` | |
| foot.`页脚` | |

`列表` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| body | posts.summary | |
| meta | posts.created_at | |
| tag | posts.tag | |
| href | posts.slug | |

`详情绑定` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| meta | posts.created_at | |
| tag | posts.tag | |
| body | posts.content | |

`标签列表` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | tags.name | |
| meta | 点击查看该标签下的文章 | |
| href | tags.slug | |

`文章条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`标签条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| tag | = | {slug} |

`管理员` =

| 行 | 用户名 | 密码 |
|----|--------|------|
| 1 | admin | marqdo |

`站点图标` =

| 路径 | 关系 | 类型 | 尺寸 | 地址 |
|------|------|------|------|------|
| "public/favicon.png" | icon | "image/png" | 32x32 | "/favicon.ico" |
| "public/favicon.svg" | icon | "image/svg+xml" | any | "/static/favicon.svg" |

`头资源` =

| 关系 | 地址 | 类型 |
|------|------|------|
| apple-touch-icon | "/static/favicon.png" | "image/png" |

`品牌图` =

| 源 | 替代 | 类 | 链接 | 宽度 | 加载 |
|----|------|----|------|------|------|
| "/static/logo.svg" | Marqdo | brand-logo | "/" | 40 | eager |

*store = > db.打开*

*首页CSS = > theme.全局*

*page = > 网页.页面 标题="Marqdo 博客" 引言="<h1>Marqdo 博客</h1><p>用 web 扩展库搭建的博客：表格装配页面、SQLite 数据、动态路由、实时终端。</p><h2 id=\"live\">实时终端</h2><p id=\"live-status\" class=\"live-pill\">connecting…</p><input id=\"live-msg\" placeholder=\"输入一条消息…\"><button id=\"live-send\" disabled>发送</button><div id=\"live-out\"></div><script src=\"/static/live.js\"></script>"*
*page = > page.组件装配 组件=`首页`*
*page = > page.主体装配 主体=`列表`*
*page = > page.排序 排序="-created_at"*
*page = > page.样式 样式=`首页CSS`*
*page = > page.头装配 表=`头资源`*
*page = > page.图片装配 表=`品牌图`*

*about = > 网页.页面 标题="关于" 引言="<h1>关于本站</h1><p>一个用 Marqdo 的 web 扩展库（ext/web）写成的博客系统。</p><p>它演示了：GFM 表格装配导航与侧栏、SQLite 驱动的主体绑定、动态路由 /post/{slug}、标签归档、以及受 session 保护的后台 CRUD。</p>"*
*about = > about.组件装配 组件=`首页`*
*about = > about.样式 样式=`首页CSS`*

*post = > 网页.页面 标题="文章"*
*post = > post.组件装配 组件=`首页`*
*post = > post.主体装配 主体=`详情绑定`*
*post = > post.查询条件 条件=`文章条件`*
*post = > post.详情 详情=True*
*post = > post.样式 样式=`首页CSS`*

*tags = > 网页.页面 标题="标签归档" 引言="<h1>标签归档</h1><p>点击标签查看该主题下的文章。</p>"*
*tags = > tags.组件装配 组件=`首页`*
*tags = > tags.主体装配 主体=`标签列表`*
*tags = > tags.链接前缀 前缀="/tag/"*
*tags = > tags.样式 样式=`首页CSS`*

*tagged = > 网页.页面 标题="标签"*
*tagged = > tagged.组件装配 组件=`首页`*
*tagged = > tagged.主体装配 主体=`列表`*
*tagged = > tagged.查询条件 条件=`标签条件`*
*tagged = > tagged.排序 排序="-created_at"*
*tagged = > tagged.样式 样式=`首页CSS`*

*app = > 网页.应用 页面=page 数据库=store 后台=True 主机="127.0.0.1" 端口=18085*
*app = > app.路由 路径="/about" 页面=about*
*app = > app.路由 路径="/post/{slug}" 页面=post*
*app = > app.路由 路径="/tags" 页面=tags*
*app = > app.路由 路径="/tag/{slug}" 页面=tagged*
*app = > app.静态 目录="public" 挂载="/static"*
*app = > app.图标 表=`站点图标`*
*app = > app.路由实时 路径="/live" 回显=True*
*app = > app.鉴权 用户表=`管理员` 会话时长=3600*
> `app`.监听