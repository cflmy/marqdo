---
title: styles/shell
description: 暗恋见君壳样式——配色与布局用表格声明。
导入 网页:ext/web/网页.mq.md
---

## 基础

暖纸感背景、深红强调（品牌向「相信爱情」），避免默认紫渐变模板。

`基础` =

| 选择器 | 属性 | 值 |
|--------|------|-----|
| :root | --ink | #2a1f1a |
| :root | --muted | #7a6560 |
| :root | --paper | #faf6f2 |
| :root | --card | #ffffff |
| :root | --line | #eadfd8 |
| :root | --accent | #9b2c2c |
| :root | --accent-ink | #fffaf7 |
| :root | --radius | 12px |
| :root | --serif | "Noto Serif SC", "Songti SC", Georgia, serif |
| :root | --sans | "Noto Sans SC", "IBM Plex Sans", system-ui, sans-serif |
| * | box-sizing | border-box |
| body | margin | "0" |
| body | font-family | var(--sans) |
| body | background | var(--paper) |
| body | color | var(--ink) |
| h1 | font-family | var(--serif) |
| h1 | font-weight | "600" |
| a | color | var(--accent) |
| .main-intro | max-width | "52rem" |
| .main-intro | margin | "1.5rem auto" |
| .main-intro | padding | "0 1.25rem" |
| .chat-box | border | 1px solid var(--line) |
| .chat-box | border-radius | var(--radius) |
| .chat-box | background | var(--card) |
| .chat-box | padding | "1rem" |
| .chat-log | min-height | "12rem" |
| .chat-log | font-family | ui-monospace, monospace |
| .chat-log | font-size | "0.9rem" |
| .chat-log | white-space | pre-wrap |

**`基础`**

## 全局

*基础表 = > 基础*
*css = > 网页.样式装配 名="anlian" 表=`基础表`*
**css**
