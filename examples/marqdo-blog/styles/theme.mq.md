---
title: styles/theme
description: 博客主题：样式即数据表格，装配即函数。
import 网页:ext/web/网页.mq.md
import text:lib/text.mq.md
---

博客主题的样式即数据表格：每个 `## 段` 导出一张 GFM 样式表（`|选择器|属性|值|`，
或用 `|媒体|选择器|属性|值|` 表达响应式），`## 全局` 用 `样式装配` 函数把它们
装配成一份完整 CSS。这符合 文档即代码 —— 样式是可读的表格，装配是显式调用。

## 基础

配色与字体变量、页面骨架。衬线标题 + 无衬线正文，暖白纸感背景。

`基础` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| :root | --ink | #1c1917 |
| :root | --ink-soft | #44403c |
| :root | --muted | #78716c |
| :root | --faint | #a8a29e |
| :root | --paper | #f7f5f2 |
| :root | --card | #ffffff |
| :root | --line | #e7e5e4 |
| :root | --accent | #4f46e5 |
| :root | --accent-2 | #7c3aed |
| :root | --accent-ink | #ffffff |
| :root | --radius | 16px |
| :root | --shadow | 0 1px 2px rgba(28,25,23,.05), 0 8px 24px rgba(28,25,23,.06) |
| :root | --shadow-lg | 0 4px 12px rgba(28,25,23,.08), 0 20px 48px rgba(28,25,23,.12) |
| :root | --serif | "Noto Serif SC", Georgia, "Songti SC", serif |
| :root | --sans | "Noto Sans SC", "IBM Plex Sans", system-ui, -apple-system, sans-serif |
| html | scroll-behavior | smooth |
| * | box-sizing | border-box |
| ::selection | background | rgba(79,70,229,.18) |
| body | margin | 0 |
| body | font-family | var(--sans) |
| body | background | var(--paper) |
| body | color | var(--ink) |
| body | display | grid |
| body | min-height | 100vh |
| body | line-height | 1.7 |
| body | grid-template-rows | auto 1fr auto |
| body.has-sidebar | grid-template-columns | 15rem 1fr |
| body.has-sidebar | grid-template-areas | "top top" "side main" "foot foot" |
| body.no-sidebar | grid-template-areas | "top" "main" "foot" |

**`基础`**

## 顶栏

吸顶导航：磨砂玻璃 + 细分隔线，Logo 处用渐变强调。

`顶栏` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| header.topnav | grid-area | top |
| header.topnav | position | sticky |
| header.topnav | top | 0 |
| header.topnav | z-index | 20 |
| header.topnav | background | rgba(255,255,255,.82) |
| header.topnav | backdrop-filter | blur(10px) |
| header.topnav | -webkit-backdrop-filter | blur(10px) |
| header.topnav | border-bottom | 1px solid var(--line) |
| header.topnav | padding | .9rem 2rem |
| header.topnav | display | flex |
| header.topnav | align-items | center |
| header.topnav | justify-content | space-between |
| ul.nav | list-style | none |
| ul.nav | margin | 0 |
| ul.nav | padding | 0 |
| ul.nav | display | flex |
| ul.nav | align-items | center |
| ul.nav | gap | .35rem |
| ul.nav li | margin | 0 |
| ul.nav a | display | inline-block |
| ul.nav a | padding | .45rem .9rem |
| ul.nav a | border-radius | 999px |
| ul.nav a | color | var(--ink-soft) |
| ul.nav a | font-weight | 500 |
| ul.nav a | text-decoration | none |
| ul.nav a | transition | background .18s ease, color .18s ease |
| ul.nav a:hover | background | rgba(79,70,229,.1) |
| ul.nav a:hover | color | var(--accent) |
| ul.nav li:first-child a | font-family | var(--serif) |
| ul.nav li:first-child a | font-size | 1.25rem |
| ul.nav li:first-child a | font-weight | 700 |
| ul.nav li:first-child a | background | linear-gradient(120deg, var(--accent), var(--accent-2)) |
| ul.nav li:first-child a | -webkit-background-clip | text |
| ul.nav li:first-child a | background-clip | text |
| ul.nav li:first-child a | -webkit-text-fill-color | transparent |
| ul.nav li:first-child a | color | transparent |
| ul.nav li:first-child a | padding-left | .2rem |
| ul.nav li:first-child a | padding-right | .2rem |

**`顶栏`**

## 侧栏

归档面板：浅底圆角卡片，链接胶囊化。

`侧栏` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| aside.side | grid-area | side |
| aside.side | padding | 2rem 1.25rem |
| aside.side | border-right | 1px solid var(--line) |
| aside.side | background | linear-gradient(180deg, #fbfaf8, #f3f1ec) |
| ul.side-nav | list-style | none |
| ul.side-nav | margin | 0 |
| ul.side-nav | padding | 0 |
| ul.side-nav | display | flex |
| ul.side-nav | flex-direction | column |
| ul.side-nav | gap | .25rem |
| ul.side-nav li | margin | 0 |
| ul.side-nav a | display | block |
| ul.side-nav a | padding | .55rem .85rem |
| ul.side-nav a | border-radius | 10px |
| ul.side-nav a | color | var(--ink-soft) |
| ul.side-nav a | text-decoration | none |
| ul.side-nav a | font-weight | 500 |
| ul.side-nav a | transition | background .15s ease, transform .15s ease, color .15s ease |
| ul.side-nav a:hover | background | #fff |
| ul.side-nav a:hover | color | var(--accent) |
| ul.side-nav a:hover | transform | translateX(4px) |
| ul.side-nav a:hover | box-shadow | 0 1px 3px rgba(28,25,23,.08) |

**`侧栏`**

## 主体

内容区留白与节奏。

`主体` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| main.main | grid-area | main |
| main.main | padding | 2.5rem 2.5rem 3.5rem |
| main.main | max-width | 76rem |
| main.main | width | 100% |
| .main-intro | margin-bottom | .5rem |
| .main-intro h1 | font-family | var(--serif) |
| .main-intro h1 | margin | 0 0 .5rem |
| .main-intro h1 | font-size | clamp(2rem, 5vw, 2.9rem) |
| .main-intro h1 | line-height | 1.2 |
| .main-intro h1 | font-weight | 700 |
| .main-intro h1 | letter-spacing | -.01em |
| .main-intro p | color | var(--muted) |
| .main-intro p | margin | 0 |
| .main-intro p | font-size | 1.05rem |

**`主体`**

## 卡片

文章卡片：白底大圆角、柔和阴影、hover 抬升，整卡可点。

`卡片` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| .content.cards | display | grid |
| .content.cards | gap | 1.5rem |
| .content.cards | margin-top | 2rem |
| .content.cards | grid-template-columns | repeat(auto-fill, minmax(17rem, 1fr)) |
| .content.cards article.card | background | var(--card) |
| .content.cards article.card | border | 1px solid var(--line) |
| .content.cards article.card | border-radius | var(--radius) |
| .content.cards article.card | padding | 1.6rem 1.7rem |
| .content.cards article.card | box-shadow | var(--shadow) |
| .content.cards article.card | transition | transform .22s ease, box-shadow .22s ease, border-color .22s ease |
| .content.cards article.card | display | flex |
| .content.cards article.card | flex-direction | column |
| .content.cards article.card | gap | .55rem |
| .content.cards article.card:hover | transform | translateY(-6px) |
| .content.cards article.card:hover | box-shadow | var(--shadow-lg) |
| .content.cards article.card:hover | border-color | rgba(79,70,229,.35) |
| .content.cards a.card-link | text-decoration | none |
| .content.cards a.card-link | color | inherit |
| .content.cards a.card-link | display | flex |
| .content.cards a.card-link | flex-direction | column |
| .content.cards a.card-link | gap | .55rem |
| .content.cards a.card-link | height | 100% |
| .content.cards a.card-link:hover | color | inherit |
| .content.cards h2 | font-family | var(--serif) |
| .content.cards h2 | margin | 0 |
| .content.cards h2 | font-size | 1.35rem |
| .content.cards h2 | line-height | 1.35 |
| .content.cards h2 | font-weight | 700 |
| .content.cards h2 | transition | color .18s ease |
| .content.cards a.card-link:hover h2 | color | var(--accent) |
| .card-meta | font-size | .82rem |
| .card-meta | color | var(--faint) |
| .card-meta | letter-spacing | .04em |
| .card-meta | text-transform | uppercase |
| .card-tag | align-self | flex-start |
| .card-tag | font-size | .78rem |
| .card-tag | color | var(--accent) |
| .card-tag | background | rgba(79,70,229,.08) |
| .card-tag | padding | .15rem .65rem |
| .card-tag | border-radius | 999px |
| .card-tag | font-weight | 600 |
| .content.cards p | color | var(--muted) |
| .content.cards p | margin | 0 |
| .content.cards p | font-size | .95rem |
| .content.cards p | flex | 1 |

**`卡片`**

## 页脚

`页脚` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| footer.foot | grid-area | foot |
| footer.foot | padding | 1.4rem 2rem |
| footer.foot | border-top | 1px solid var(--line) |
| footer.foot | color | var(--muted) |
| footer.foot | background | #fff |
| footer.foot | font-size | .9rem |
| footer.foot | display | flex |
| footer.foot | justify-content | space-between |
| footer.foot | align-items | center |
| footer.foot | flex-wrap | wrap |
| footer.foot | gap | .5rem |
| ul.foot-nav | list-style | none |
| ul.foot-nav | margin | 0 |
| ul.foot-nav | padding | 0 |
| ul.foot-nav | display | flex |
| ul.foot-nav | gap | 1rem |
| ul.foot-nav | flex-wrap | wrap |
| ul.foot-nav li | margin | 0 |
| ul.foot-nav a | color | var(--muted) |
| ul.foot-nav a | text-decoration | none |
| ul.foot-nav a:hover | color | var(--accent) |

**`页脚`**

## 表单

后台表单统一风格。

`表单` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| .site-form | margin-top | 1.75rem |
| .site-form | max-width | 34rem |
| .site-form | background | var(--card) |
| .site-form | border | 1px solid var(--line) |
| .site-form | border-radius | var(--radius) |
| .site-form | padding | 1.8rem |
| .site-form | box-shadow | var(--shadow) |
| .site-form form | display | grid |
| .site-form form | gap | 1rem |
| .site-form label | display | grid |
| .site-form label | gap | .35rem |
| .site-form label | font-size | .92rem |
| .site-form label | font-weight | 600 |
| .site-form label | color | var(--ink-soft) |
| .site-form input | padding | .65rem .8rem |
| .site-form input | border | 1px solid var(--line) |
| .site-form input | border-radius | 10px |
| .site-form input | font | inherit |
| .site-form input | background | #fdfcfb |
| .site-form input | transition | border-color .15s ease, box-shadow .15s ease |
| .site-form textarea | padding | .65rem .8rem |
| .site-form textarea | border | 1px solid var(--line) |
| .site-form textarea | border-radius | 10px |
| .site-form textarea | font | inherit |
| .site-form textarea | background | #fdfcfb |
| .site-form textarea | transition | border-color .15s ease, box-shadow .15s ease |
| .site-form select | padding | .65rem .8rem |
| .site-form select | border | 1px solid var(--line) |
| .site-form select | border-radius | 10px |
| .site-form select | font | inherit |
| .site-form select | background | #fdfcfb |
| .site-form select | transition | border-color .15s ease, box-shadow .15s ease |
| .site-form input:focus | outline | none |
| .site-form input:focus | border-color | var(--accent) |
| .site-form input:focus | box-shadow | 0 0 0 3px rgba(79,70,229,.12) |
| .site-form textarea:focus | outline | none |
| .site-form textarea:focus | border-color | var(--accent) |
| .site-form textarea:focus | box-shadow | 0 0 0 3px rgba(79,70,229,.12) |
| .site-form select:focus | outline | none |
| .site-form select:focus | border-color | var(--accent) |
| .site-form select:focus | box-shadow | 0 0 0 3px rgba(79,70,229,.12) |
| .site-form input[readonly] | background | #f5f5f4 |
| .site-form input[readonly] | color | var(--muted) |
| .site-form textarea | min-height | 10rem |
| .site-form textarea | resize | vertical |
| .site-form .err | color | #b91c1c |
| .site-form .err | font-size | .85rem |
| .site-form .actions | display | flex |
| .site-form .actions | gap | .75rem |
| .site-form .actions | align-items | center |
| .site-form .actions | flex-wrap | wrap |
| .site-form .actions | padding-top | .25rem |
| .site-form button | background | linear-gradient(120deg, var(--accent), var(--accent-2)) |
| .site-form button | color | var(--accent-ink) |
| .site-form button | border | 0 |
| .site-form button | padding | .6rem 1.3rem |
| .site-form button | border-radius | 999px |
| .site-form button | cursor | pointer |
| .site-form button | font-weight | 600 |
| .site-form button | font-size | .95rem |
| .site-form button | transition | opacity .15s ease, transform .15s ease, box-shadow .15s ease |
| .site-form button:hover | opacity | .92 |
| .site-form button:hover | transform | translateY(-1px) |
| .site-form button:hover | box-shadow | 0 6px 18px rgba(79,70,229,.3) |
| .site-form a.btn | display | inline-block |
| .site-form a.btn | padding | .6rem 1.3rem |
| .site-form a.btn | border-radius | 999px |
| .site-form a.btn | border | 1px solid var(--line) |
| .site-form a.btn | color | var(--ink-soft) |
| .site-form a.btn | text-decoration | none |
| .site-form a.btn | font-weight | 500 |
| .site-form a.btn | font-size | .95rem |
| .site-form .meta | color | var(--muted) |
| .site-form .meta | font-size | .9rem |
| .site-form .admin-list | margin | 0 |
| .site-form .admin-list | padding | 0 |
| .site-form .admin-list | list-style | none |
| .site-form .admin-list | display | grid |
| .site-form .admin-list | gap | .5rem |
| .site-form .admin-list li | background | #fdfcfb |
| .site-form .admin-list li | border | 1px solid var(--line) |
| .site-form .admin-list li | border-radius | 10px |
| .site-form .admin-list li | padding | .6rem .9rem |
| .site-form .admin-list li | display | flex |
| .site-form .admin-list li | justify-content | space-between |
| .site-form .admin-list li | align-items | center |
| .site-form .admin-list li | gap | 1rem |
| .site-form .admin-list a | color | var(--accent) |
| .site-form .admin-list a | text-decoration | none |
| .site-form .admin-list a | font-size | .9rem |

**`表单`**

## 响应式

窄屏收为单栏，导航换行。

`响应式` =

| 媒体 | 选择器 | 属性 | 值 |
|------|--------|------|-----|
| (max-width: 860px) | body.has-sidebar | grid-template-columns | 1fr |
| (max-width: 860px) | body.has-sidebar | grid-template-areas | "top" "main" "side" "foot" |
| (max-width: 860px) | aside.side | border-right | 0 |
| (max-width: 860px) | aside.side | border-bottom | 1px solid var(--line) |
| (max-width: 860px) | aside.side | padding | 1rem 2rem |
| (max-width: 860px) | ul.side-nav | flex-direction | row |
| (max-width: 860px) | ul.side-nav | flex-wrap | wrap |
| (max-width: 860px) | main.main | padding | 1.75rem 1.25rem 2.5rem |
| (max-width: 860px) | header.topnav | padding | .75rem 1.25rem |
| (max-width: 860px) | footer.foot | padding | 1.2rem 1.25rem |
| (max-width: 860px) | .content.cards | grid-template-columns | 1fr |
| (max-width: 520px) | ul.nav | flex-wrap | wrap |

**`响应式`**

## 全局

用 样式装配 函数逐个把段落表装配成 CSS，再拼接成完整样式表。

*基础表 = > 基础*
*顶栏表 = > 顶栏*
*侧栏表 = > 侧栏*
*主体表 = > 主体*
*卡片表 = > 卡片*
*页脚表 = > 页脚*
*表单表 = > 表单*
*响应式表 = > 响应式*

*css基础 = > 网页.样式装配 名="基础" 表=`基础表`*
*css顶栏 = > 网页.样式装配 名="顶栏" 表=`顶栏表`*
*css侧栏 = > 网页.样式装配 名="侧栏" 表=`侧栏表`*
*css主体 = > 网页.样式装配 名="主体" 表=`主体表`*
*css卡片 = > 网页.样式装配 名="卡片" 表=`卡片表`*
*css页脚 = > 网页.样式装配 名="页脚" 表=`页脚表`*
*css表单 = > 网页.样式装配 名="表单" 表=`表单表`*
*css响应式 = > 网页.样式装配 名="响应式" 表=`响应式表`*

`css段` =

| css |
|-----|
| `css基础` |
| `css顶栏` |
| `css侧栏` |
| `css主体` |
| `css卡片` |
| `css页脚` |
| `css表单` |
| `css响应式` |

*css = > text.str_join xs=`css段` sep=""*
**css**
