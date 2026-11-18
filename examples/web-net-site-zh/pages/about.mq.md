---
markup: occlusion
type: web
view: main
route: /about
title: 关于
import:
  组件: ext/web/网页/组件/索引.mq.md
  布局: ../components/index.mq.md
  文件: lib/文件.mq.md
---

本 Document 用元数据显式声明 /about 路径。说明留在终端 Markdown，导航、侧栏和页脚归共享布局。解释页不查询数据库、不配置服务，也不会隐式注册其他业务路径。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| 结果 | list | 共享站点框架中的关于说明 |

启动阶段只装配结构，授予 fs.read 读取 Markdown 叶子。沿布局导入可以独立修改一个区域，无需在这里重写整页壳。内容是普通 Markdown，不把源码手册的执行标记混进用户页面。 路由来自本页声明，首次阅读即可判断入口与内容来源。

我们 **文本 = [读取本页的终端说明](文件.读文本(路径="components/content/about.md"))** 和 **正文 = [保留简单 Markdown 内容](组件.正文(文本=`文本`, 槽="intro"))**。
最后返回 *[放进共享递归布局](布局.框架(正文=`正文`))*。
