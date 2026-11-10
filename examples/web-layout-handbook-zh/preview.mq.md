---
markup: occlusion
title: 可直接调用的布局预览
import:
  home: ./components/index.mq.md
  网页: ext/web/网页.mq.md
---

预览从站点组件根开始，不拼接另一份页面。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | 与原有运行输出一致的 HTML 文本 |

本函数既可以用 marqdo call 直接取得 HTML 返回值，也保留 marqdo run 原有的预览输出。组件内容相对站点根目录读取，因此从仓库根调用时应将 --fs-root 指向 examples/web-layout-handbook-zh，并授予 fs.read、plugin 和 output.write。预览不会开启网络监听，也不修改内容文件；线上服务与本函数共享节点渲染器，页脚左右区域和嵌套链接列应保持一致。检查返回的 HTML 不能代替浏览器中的布局和交互验证。

我们 **nodes = [沿组件入口装配全部布局](home.节点())**，再 **html = [使用线上相同的节点渲染器](网页.渲染(节点=`nodes`, 标题="Marqdo 布局实践"))**。
我们 **[为原有运行预览输出页面](打印(内容=`html`))**，随后直接返回 *html*。
