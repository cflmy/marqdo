---
title: 中文站点示例
description: >-
  与 examples/web-site 对等的 WWW Demo 样站：表驱动全站；作者业务 JS=0；作者 .css=0。
导入 网页:ext/web/网页.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import desk:components/desk.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

本文件既是站点说明，也是可运行应用。

`home` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side_panel` |
| foot.`foot` | shell.`footer` |

后台壳：去掉前台侧栏，顶栏品牌改为「后台」。

`desk_shell` =

| 组件 | 样式 |
|------|------|
| desk.`nav` | shell.`topnav` |
| desk.`foot` | shell.`footer` |

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.summary | shell.`card_body` |
| href | articles.`articles`.slug | |

`detail` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.body | |

`by_slug` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`首页引言` =

| 属性 | 值 | 样式 |
|------|-----|------|
| 眉题 | WWW Demo · 表即全栈 | shell.`kicker` |
| 标题 | Marqdo | shell.`intro_title` |
| 导语 | "一份可执行 Markdown 声明路由、表单、后台与样式——作者零业务 JavaScript、零手写 .css。点击卡片打开全文，再按下列三步动手试。" | shell.`lede` |
| 标签 | 0 作者 JS | shell.`claim` |
| 标签 | 0 作者 .css | shell.`claim` |
| 标签 | GFM 表 | shell.`claim` |
| 步骤 | "点击任意卡片 — /post/{slug} 用同一张 articles 表打开详情。" | shell.`step` |
| 步骤 | "打开 [写文章](/new)，空标题提交（规则表校验）。" | shell.`step` |
| 步骤 | "点顶栏 **登录**（admin / demo），再进 **后台** — 门禁管理台：创建表单 + 列表。" | shell.`step` |

`关于引言` =

| 属性 | 值 | 样式 |
|------|-----|------|
| 眉题 | 关于 | shell.`kicker` |
| 标题 | 关于 | shell.`intro_title` |
| 导语 | "文档与站点同一 .mq.md 制品。版式与主题是 styles/shell.mq.md 里的 GFM 样式表——无需手写 CSS 或业务 JavaScript。" | shell.`lede` |

`写文章引言` =

| 属性 | 值 | 样式 |
|------|-----|------|
| 眉题 | 表单 · 试校验 | shell.`kicker` |
| 标题 | 写文章 | shell.`intro_title` |
| 导语 | 空标题提交应失败（规则表，非客户端 JS）；填写标题、slug 与摘要后与后台、卡片网格共享一行。 | shell.`lede` |

`后台引言` =

| 属性 | 值 | 样式 |
|------|-----|------|
| 眉题 | 管理台 · 已进入后台 | shell.`kicker` |
| 标题 | 后台 | shell.`intro_title` |
| 导语 | "这就是管理台（需登录）：与前台共用 articles schema，装配成列表 + 创建表单——不是另写一套 SPA。顶栏「站点」回前台，「退出」结束会话。" | shell.`lede` |
| 标签 | 登录门禁 | shell.`claim` |
| 标签 | 表声明 | shell.`claim` |
| 标签 | "创建 + 列表" | shell.`claim` |

`文章引言` =

| 属性 | 值 | 样式 |
|------|-----|------|
| 眉题 | 文章 · 表绑定详情 | shell.`kicker` |

`article_fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | 标题 | text | true | |
| slug | slug | text | true | |
| summary | 摘要 | textarea | false | |
| body | 正文 | textarea | false | |

`article_rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| title | max:120 | 标题太长 |
| slug | required | slug 不能为空 |
| slug | max:80 | slug 太长 |
| summary | max:500 | 摘要太长 |
| body | max:8000 | 正文太长 |

`home_meta` =

| key | value |
|-----|-------|
| description | 表即全栈 — Marqdo 网页示例；作者零业务 JavaScript。 |
| og:type | website |
| og:title | Marqdo 中文站 |

`fonts` =

| 关系 | 地址 | 跨域 |
|------|------|------|
| preconnect | https://fonts.googleapis.com | |
| preconnect | https://fonts.gstatic.com | anonymous |
| stylesheet | https://fonts.googleapis.com/css2?family=Great+Vibes&family=Noto+Sans+SC:wght@300;400;500;700&family=Noto+Serif+SC:wght@400;600&display=swap | |

**store = > db.open**
**site_css = > shell.css**

**page = > 网页.页面 标题="Marqdo 中文站" 壳样式="off"**
**page = > page.引言装配 引言=`首页引言`**
**page = > page.元数据 元数据=home_meta**
**page = > page.头装配 表=fonts**
**page = > page.样式 样式=site_css**
**page = > page.组件装配 组件=home**
**page = > page.主体装配 主体=index**
**page = > page.链接前缀 前缀="/post/"**

**about = > 网页.页面 标题="关于 · Marqdo 中文站" 壳样式="off"**
**about = > about.引言装配 引言=`关于引言`**
**about = > about.头装配 表=fonts**
**about = > about.样式 样式=site_css**
**about = > about.组件装配 组件=home**

**article_form = > 网页.表单 表="articles" 动作="插入"**
**article_form = > article_form.字段 字段=article_fields**
**article_form = > article_form.规则 规则=article_rules**

**new = > 网页.页面 标题="写文章 · Marqdo 中文站" 壳样式="off"**
**new = > new.引言装配 引言=`写文章引言`**
**new = > new.头装配 表=fonts**
**new = > new.样式 样式=site_css**
**new = > new.组件装配 组件=home**
**new = > new.表单装配 id="article" 表单=article_form**

**desk = > 网页.页面 标题="后台 · Marqdo 中文站" 壳样式="off"**
**desk = > desk.壳HTML 体类="is-desk"**
**desk = > desk.引言装配 引言=`后台引言`**
**desk = > desk.头装配 表=fonts**
**desk = > desk.样式 样式=site_css**
**desk = > desk.组件装配 组件=desk_shell**
**desk = > desk.表单装配 id="article" 表单=article_form**
**desk = > desk.主体装配 主体=index**
**desk = > desk.链接前缀 前缀="/post/"**

**post = > 网页.页面 标题="文章 · Marqdo 中文站" 壳样式="off"**
**post = > post.引言装配 引言=`文章引言`**
**post = > post.头装配 表=fonts**
**post = > post.样式 样式=site_css**
**post = > post.组件装配 组件=home**
**post = > post.主体装配 主体=detail**
**post = > post.查询条件 条件=by_slug**
**post = > post.详情 详情=True**

演示账号（用户表声明；宿主会话鉴权，无作者 JS）。

`管理员` =

| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | demo | admin |

**app = > 网页.应用 页面=page 数据库=store 后台=False 主机="127.0.0.1" 端口=18082 壳样式="off" 登录回跳="/admin" 登出回跳="/"**
**app = > app.路由 路径="/about" 页面=about**
**app = > app.路由 路径="/new" 页面=new**
**app = > app.路由 路径="/admin" 页面=desk**
**app = > app.路由 路径="/post/{slug}" 页面=post**
**app = > app.鉴权 用户表=`管理员` 会话时长=3600 登录路径="/login" 登录回跳="/admin" 登出回跳="/"**
**app = > app.门禁 路径="/admin" 角色="admin" 匹配="prefix" 拒绝="redirect" 排除="/login"**
> `app`.监听
