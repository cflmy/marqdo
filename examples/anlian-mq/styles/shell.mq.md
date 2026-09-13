---
title: styles/shell
description: 暗恋见君壳样式——对照 Django 原站粉玻璃 + 动效，用表格声明。
导入 网页:ext/web/网页.mq.md
import text:lib/text.mq.md
---

对照原站 `config/static/front/css/{base,feed_common,footer,index/hero}.css`：
固定背景图、毛玻璃顶栏、Logo 呼吸、Hero 漂浮、粉玻璃卡片与页脚。

## 基础

品牌色与全站底：固定摄影背景 + 粉玻璃变量。

`基础` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| :root | --ink | #333333 |
| :root | --ink-strong | #111111 |
| :root | --muted | #666666 |
| :root | --paper | #f8f9fa |
| :root | --glass | rgba(255,255,255,0.28) |
| :root | --glass-strong | rgba(255,255,255,0.42) |
| :root | --glass-border | rgba(255,255,255,0.5) |
| :root | --accent | #fdbbdb |
| :root | --accent-deep | #f0a8c4 |
| :root | --accent-text | #a85878 |
| :root | --accent-muted | #c97a96 |
| :root | --accent-bg | rgba(253,189,219,0.48) |
| :root | --hero-pink | #ffd0d0 |
| :root | --radius | 14px |
| :root | --sans | "Noto Sans SC", "IBM Plex Sans", system-ui, sans-serif |
| * | box-sizing | border-box |
| html | scroll-behavior | smooth |
| body | margin | "0" |
| body | min-height | 100vh |
| body | font-family | var(--sans) |
| body | color | var(--ink) |
| body | line-height | "1.6" |
| body | background-color | var(--paper) |
| body | background-image | url("/static/bg-body-960.webp") |
| body | background-repeat | no-repeat |
| body | background-size | cover |
| body | background-position | center center |
| body | background-attachment | fixed |
| body | overflow-x | hidden |
| a | color | var(--accent-text) |
| a | text-decoration | none |
| a:hover | color | var(--accent-deep) |
| ::selection | background | rgba(253,189,219,0.45) |
| main.main | padding | 0 0 3rem |
| .main-intro | max-width | 72rem |
| .main-intro | margin | 0 auto |
| .main-intro | padding | 1.25rem 1.5rem 0 |

*`基础`*

## 动效

原站 `pulse`（Logo）与 `heroImageFloat`（主图）；尊重减动偏好。

`动效` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| @keyframes pulse 0% | transform | scale(1) |
| @keyframes pulse 50% | transform | scale(1.05) |
| @keyframes pulse 100% | transform | scale(1) |
| @keyframes heroImageFloat 0% | transform | scale(1) |
| @keyframes heroImageFloat 50% | transform | scale(1.04) |
| @keyframes heroImageFloat 100% | transform | scale(1) |
| @keyframes cardRise 0% | opacity | "0" |
| @keyframes cardRise 0% | transform | translateY(10px) |
| @keyframes cardRise 100% | opacity | "1" |
| @keyframes cardRise 100% | transform | translateY(0) |

*`动效`*

## 顶栏

毛玻璃导航；首项品牌带 Logo 呼吸动画。

`顶栏` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| header.topnav | grid-area | top |
| header.topnav | position | sticky |
| header.topnav | top | "0" |
| header.topnav | z-index | "1000" |
| header.topnav | width | 100% |
| header.topnav | padding | 0.75rem 1.25rem |
| header.topnav | background | rgba(255,255,255,0.2) |
| header.topnav | backdrop-filter | blur(24px) |
| header.topnav | -webkit-backdrop-filter | blur(24px) |
| header.topnav | border-bottom | 1px solid rgba(255,255,255,0.2) |
| header.topnav | box-shadow | inset 0 0 5px rgba(255,255,255,0.1) |
| ul.nav | list-style | none |
| ul.nav | margin | "0" |
| ul.nav | padding | "0" |
| ul.nav | display | flex |
| ul.nav | flex-wrap | wrap |
| ul.nav | align-items | center |
| ul.nav | gap | 0.35rem 0.5rem |
| ul.nav a | display | inline-flex |
| ul.nav a | align-items | center |
| ul.nav a | gap | 0.45rem |
| ul.nav a | padding | 0.4rem 0.85rem |
| ul.nav a | border-radius | 999px |
| ul.nav a | color | var(--ink) |
| ul.nav a | font-weight | "600" |
| ul.nav a | transition | color 0.2s ease, transform 0.2s ease, background 0.2s ease |
| ul.nav a:hover | color | var(--accent-text) |
| ul.nav a:hover | transform | translateY(-2px) |
| ul.nav a:hover | background | rgba(253,189,219,0.22) |
| ul.nav li:first-child a | font-size | 1.05rem |
| ul.nav li:first-child a | font-weight | "700" |
| ul.nav li:first-child a | letter-spacing | 0.04em |
| ul.nav li:first-child a | padding-left | 0.35rem |
| ul.nav li:first-child a::before | content | "" |
| ul.nav li:first-child a::before | width | 28px |
| ul.nav li:first-child a::before | height | 28px |
| ul.nav li:first-child a::before | background | url("/static/logo.svg") center / contain no-repeat |
| ul.nav li:first-child a::before | animation | pulse 3s ease-in-out infinite |
| ul.nav li:first-child a:hover | transform | none |
| ul.nav li:first-child a:hover | background | transparent |

*`顶栏`*

## Hero

首页双栏英雄区：大标题粉强调 + 主图漂浮。

`Hero` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| .hero | max-width | 72rem |
| .hero | margin | 0 auto 1.5rem |
| .hero | padding | 1.5rem 0.5rem 0.5rem |
| .hero-content | display | grid |
| .hero-content | grid-template-columns | minmax(0,1fr) minmax(0,1fr) |
| .hero-content | align-items | center |
| .hero-content | gap | 2rem |
| .hero-left | display | flex |
| .hero-left | flex-direction | column |
| .hero-left | align-items | flex-start |
| .hero-brand | display | inline-flex |
| .hero-brand | align-items | center |
| .hero-brand | gap | 0.55rem |
| .hero-brand | margin | 0 0 1rem |
| .hero-brand | font-size | 1.35rem |
| .hero-brand | font-weight | "700" |
| .hero-brand | letter-spacing | 0.08em |
| .hero-brand | text-transform | uppercase |
| .hero-brand | color | var(--ink-strong) |
| .hero-logo | width | 30px |
| .hero-logo | height | 30px |
| .hero-logo | animation | pulse 3s ease-in-out infinite |
| .hero-pink | color | var(--hero-pink) |
| .main-text | margin | 0 0 1.25rem |
| .main-text | font-size | clamp(2.4rem, 5.5vw, 5.2rem) |
| .main-text | font-weight | "800" |
| .main-text | line-height | "1.15" |
| .main-text | color | var(--ink-strong) |
| .sub-text | margin | 0 0 1.75rem |
| .sub-text | max-width | 34rem |
| .sub-text | font-size | 1.05rem |
| .sub-text | letter-spacing | 0.02em |
| .sub-text | color | rgba(60,60,60,0.85) |
| .hero-cta | display | flex |
| .hero-cta | flex-wrap | wrap |
| .hero-cta | gap | 0.65rem |
| .hero-btn | display | inline-flex |
| .hero-btn | align-items | center |
| .hero-btn | padding | 0.55rem 1.2rem |
| .hero-btn | border-radius | 999px |
| .hero-btn | font-weight | "600" |
| .hero-btn | color | #fff |
| .hero-btn | background | linear-gradient(120deg, var(--accent-deep), var(--accent)) |
| .hero-btn | box-shadow | 0 6px 18px rgba(253,189,219,0.45) |
| .hero-btn | transition | transform 0.2s ease, box-shadow 0.2s ease |
| .hero-btn:hover | transform | translateY(-2px) |
| .hero-btn:hover | color | #fff |
| .hero-btn:hover | box-shadow | 0 10px 24px rgba(253,189,219,0.55) |
| .hero-btn.ghost | background | rgba(255,255,255,0.35) |
| .hero-btn.ghost | color | var(--accent-text) |
| .hero-btn.ghost | border | 1px solid var(--glass-border) |
| .hero-btn.ghost | box-shadow | none |
| .hero-btn.ghost:hover | background | rgba(255,255,255,0.55) |
| .hero-btn.ghost:hover | color | var(--accent-text) |
| .hero-right | display | flex |
| .hero-right | align-items | center |
| .hero-right | justify-content | center |
| .hero-art | display | block |
| .hero-art | width | 85% |
| .hero-art | max-width | 420px |
| .hero-art | height | auto |
| .hero-art | filter | drop-shadow(0 0 20px rgba(255,208,208,0.45)) |
| .hero-art | animation | heroImageFloat 30s linear infinite |
| .hero-art | pointer-events | none |
| .home-feed-head | max-width | 52rem |
| .home-feed-head | margin | 0.5rem auto 0 |
| .home-feed-head | padding | 0 1.25rem |
| .home-feed-head h2 | margin | 0 0 0.35rem |
| .home-feed-head h2 | font-size | 1.35rem |
| .home-feed-head h2 | color | var(--accent-text) |
| .home-feed-head p | margin | "0" |
| .home-feed-head p | color | var(--muted) |
| .home-feed-head p | font-size | 0.92rem |
| .page-stage | max-width | 52rem |
| .page-stage | margin | 1.25rem auto 0 |
| .page-stage | padding | 1.25rem 1.35rem |
| .page-stage | border-radius | var(--radius) |
| .page-stage | background | var(--glass) |
| .page-stage | backdrop-filter | blur(14px) |
| .page-stage | -webkit-backdrop-filter | blur(14px) |
| .page-stage | border | 1px solid var(--glass-border) |
| .page-stage h1 | margin | 0 0 0.35rem |
| .page-stage h1 | font-size | 1.6rem |
| .page-stage h1 | color | var(--ink-strong) |
| .page-stage p | margin | "0" |
| .page-stage p | color | var(--muted) |

*`Hero`*

## 卡片

粉玻璃帖子/新闻卡：悬停上浮（对齐 `.home-post-card`）。

`卡片` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| .content.cards | max-width | 52rem |
| .content.cards | margin | 1rem auto 0 |
| .content.cards | padding | 0 1.25rem |
| .content.cards | display | flex |
| .content.cards | flex-direction | column |
| .content.cards | gap | 0.65rem |
| article.card | border-radius | var(--radius) |
| article.card | background | rgba(255,255,255,0.32) |
| article.card | backdrop-filter | blur(12px) |
| article.card | -webkit-backdrop-filter | blur(12px) |
| article.card | border | 1px solid rgba(255,255,255,0.42) |
| article.card | box-shadow | 0 4px 16px rgba(253,189,219,0.08) |
| article.card | transition | transform 0.2s ease, box-shadow 0.2s ease, background 0.2s ease |
| article.card | animation | cardRise 0.45s ease both |
| article.card:nth-child(2) | animation-delay | 0.05s |
| article.card:nth-child(3) | animation-delay | 0.1s |
| article.card:nth-child(4) | animation-delay | 0.15s |
| article.card:hover | transform | translateY(-2px) |
| article.card:hover | background | rgba(255,255,255,0.42) |
| article.card:hover | box-shadow | 0 8px 24px rgba(253,189,219,0.18) |
| a.card-link | display | grid |
| a.card-link | grid-template-columns | auto 1fr |
| a.card-link | grid-template-areas | "meta title" "meta body" |
| a.card-link | gap | 0.35rem 1rem |
| a.card-link | align-items | start |
| a.card-link | padding | 0.85rem 1rem |
| a.card-link | color | inherit |
| a.card-link | text-decoration | none |
| .card-meta | grid-area | meta |
| .card-meta | align-self | center |
| .card-meta | padding | 0.2rem 0.55rem |
| .card-meta | border-radius | 6px |
| .card-meta | font-size | 0.6875rem |
| .card-meta | font-weight | "600" |
| .card-meta | color | var(--accent-text) |
| .card-meta | background | rgba(253,189,219,0.28) |
| .card-meta | white-space | nowrap |
| .card-meta | max-width | 5rem |
| .card-meta | overflow | hidden |
| .card-meta | text-overflow | ellipsis |
| article.card h2 | grid-area | title |
| article.card h2 | margin | "0" |
| article.card h2 | font-size | 0.95rem |
| article.card h2 | font-weight | "600" |
| article.card h2 | color | #222 |
| article.card h2 | line-height | "1.35" |
| article.card p | grid-area | body |
| article.card p | margin | "0" |
| article.card p | font-size | 0.82rem |
| article.card p | color | #666 |
| article.card p | line-height | "1.45" |
| article.article | max-width | 42rem |
| article.article | margin | 1.5rem auto |
| article.article | padding | 1.5rem 1.6rem |
| article.article | border-radius | var(--radius) |
| article.article | background | rgba(255,255,255,0.55) |
| article.article | backdrop-filter | blur(16px) |
| article.article | -webkit-backdrop-filter | blur(16px) |
| article.article | border | 1px solid var(--glass-border) |
| article.article | box-shadow | 0 8px 28px rgba(253,189,219,0.12) |
| .article-meta | color | var(--accent-text) |
| .article-meta | font-size | 0.85rem |
| .article-meta | font-weight | "600" |
| .article-meta | margin-bottom | 0.5rem |
| .article-title | margin | 0 0 1rem |
| .article-title | font-size | 1.75rem |
| .article-title | color | var(--ink-strong) |
| .article-body | color | var(--ink) |
| .article-body | line-height | "1.75" |
| .site-form | max-width | 34rem |
| .site-form | margin | 1.5rem auto |
| .site-form | padding | 1.5rem |
| .site-form | border-radius | var(--radius) |
| .site-form | background | rgba(255,255,255,0.45) |
| .site-form | backdrop-filter | blur(14px) |
| .site-form | border | 1px solid var(--glass-border) |
| .chat-box | max-width | 42rem |
| .chat-box | margin | 1rem auto |
| .chat-box | padding | 1rem 1.15rem |
| .chat-box | border-radius | var(--radius) |
| .chat-box | background | rgba(255,255,255,0.4) |
| .chat-box | backdrop-filter | blur(12px) |
| .chat-box | border | 1px solid var(--glass-border) |
| .chat-log | min-height | 12rem |
| .chat-log | font-family | ui-monospace, monospace |
| .chat-log | font-size | 0.9rem |
| .chat-log | white-space | pre-wrap |
| .search-bar | display | flex |
| .search-bar | gap | 0.5rem |
| .search-bar | margin-top | 0.85rem |
| .search-bar input | flex | "1" |
| .search-bar input | padding | 0.55rem 0.8rem |
| .search-bar input | border-radius | 999px |
| .search-bar input | border | 1px solid rgba(253,189,219,0.55) |
| .search-bar input | background | rgba(255,255,255,0.65) |
| .search-bar button | padding | 0.55rem 1.1rem |
| .search-bar button | border | "0" |
| .search-bar button | border-radius | 999px |
| .search-bar button | background | var(--accent) |
| .search-bar button | color | var(--accent-text) |
| .search-bar button | font-weight | "600" |
| .search-bar button | cursor | pointer |

*`卡片`*

## 页脚

双栏粉玻璃页脚。

`页脚` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| footer.foot | grid-area | foot |
| footer.foot | margin-top | 2rem |
| footer.foot | padding | 2rem 1.5rem 2.5rem |
| footer.foot | background | rgba(255,255,255,0.28) |
| footer.foot | backdrop-filter | blur(14px) |
| footer.foot | -webkit-backdrop-filter | blur(14px) |
| footer.foot | border-top | 1px solid rgba(255,255,255,0.45) |
| footer.foot | box-shadow | inset 0 1px 0 rgba(255,255,255,0.35) |
| ul.foot-nav | list-style | none |
| ul.foot-nav | margin | 0 auto |
| ul.foot-nav | padding | "0" |
| ul.foot-nav | max-width | 52rem |
| ul.foot-nav | display | flex |
| ul.foot-nav | flex-wrap | wrap |
| ul.foot-nav | justify-content | center |
| ul.foot-nav | gap | 0.75rem 1.5rem |
| ul.foot-nav a | color | #444 |
| ul.foot-nav a | font-weight | "500" |
| ul.foot-nav a:hover | color | var(--accent-text) |

*`页脚`*

## 响应式

平板收窄标题；手机 Hero 单栏。

`响应式` =

| 媒体 | 选择器 | 属性 | 值 |
|------|--------|------|-----|
| (max-width: 1024px) | .hero-content | gap | 1.25rem |
| (max-width: 1024px) | .main-text | font-size | clamp(2.2rem, 4.5vw, 3.6rem) |
| (max-width: 1024px) | .hero-art | width | min(90%, 340px) |
| (max-width: 767px) | .hero-content | grid-template-columns | 1fr |
| (max-width: 767px) | .hero-left | align-items | center |
| (max-width: 767px) | .hero-left | text-align | center |
| (max-width: 767px) | .sub-text | text-align | center |
| (max-width: 767px) | .hero-cta | justify-content | center |
| (max-width: 767px) | .hero-right | order | "2" |
| (max-width: 767px) | .hero-left | order | "1" |
| (max-width: 767px) | header.topnav | padding | 0.65rem 0.85rem |
| (max-width: 767px) | ul.nav | gap | 0.25rem |
| (prefers-reduced-motion: reduce) | .hero-art | animation | none |
| (prefers-reduced-motion: reduce) | .hero-logo | animation | none |
| (prefers-reduced-motion: reduce) | ul.nav li:first-child a::before | animation | none |
| (prefers-reduced-motion: reduce) | article.card | animation | none |

*`响应式`*

## 全局

**基础表 = > 基础**
**动效表 = > 动效**
**顶栏表 = > 顶栏**
**Hero表 = > Hero**
**卡片表 = > 卡片**
**页脚表 = > 页脚**
**响应式表 = > 响应式**

**css基础 = > 网页.样式装配 名="基础" 表=`基础表`**
**css动效 = > 网页.样式装配 名="动效" 表=`动效表`**
**css顶栏 = > 网页.样式装配 名="顶栏" 表=`顶栏表`**
**cssHero = > 网页.样式装配 名="Hero" 表=`Hero表`**
**css卡片 = > 网页.样式装配 名="卡片" 表=`卡片表`**
**css页脚 = > 网页.样式装配 名="页脚" 表=`页脚表`**
**css响应式 = > 网页.样式装配 名="响应式" 表=`响应式表`**

`css段` =

| css |
|-----|
| `css基础` |
| `css动效` |
| `css顶栏` |
| `cssHero` |
| `css卡片` |
| `css页脚` |
| `css响应式` |

**css = > text.str_join xs=`css段` sep=""**
*css*
