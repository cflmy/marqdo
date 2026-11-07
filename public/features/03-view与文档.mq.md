---
markup: occlusion
title: view 与用户文档站
description: 本地浏览与静态导出由解释器生成
---

# main

版本 2.5.0：`marqdo view` 与 `view output` 共用暖白金色阅读界面，新 Marqdo 标志、
衬线正文、花枝装饰、导航搜索与底部悬浮工具栏。宽屏将真实执行结果沿左右页边引出，
窄屏将标注放回对应段落下方；点击标注可查看完整捕获值，调用帧和循环轮次仍可切换。

静态导出内嵌装饰素材、KaTeX 与数学字体，断网也能阅读公式。播放按钮在实时 view 中
运行文档；静态页面保留导出结果，并提示如何启动实时 view。普通文档重跑后，正文、
变量和执行标注使用同一次执行捕获同步刷新。

本地浏览：`marqdo view public`

静态导出：`marqdo view output public -o public`

Browse Structure 保真叙述、高亮可执行标记，并挂载运行结果（变量值、分支命中臂、调用/循环帧）。
用户站与 live view 同源渲染；CI 发布到 `gh-pages`。

> 打印 内容=文档站：由 Marqdo 解释器生成，不是手写 HTML。
