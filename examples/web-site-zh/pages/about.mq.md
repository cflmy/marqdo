---
markup: occlusion
type: web
view: main
route: /about
title: 关于 · Marqdo 中文站
import:
  布局: ../components/index.mq.md
  引言: ../components/intro.mq.md
  文章: ../components/articles.mq.md
  组件: ext/web/网页/组件/索引.mq.md
  表单: ext/data/表单.mq.md
  表: lib/表.mq.md
---

本 Document 显式声明 /about，介绍留在终端 Markdown，框架按职责共享。请求资源继续拥有自己的数据和提交责任。 页面启动函数不初始化数据库或开启另一个监听进程。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 共享框架内的本页内容与请求资源 |

装配授予 fs.read 以读取终端说明，构造描述仍是纯计算。这份解释页不需要查询或表单，修改一个布局区域应沿导入找到对应子模块，不在本文件重新拼页面壳。

结构保持稳定，请求资源观察当前数据与身份。父层决定位置，终端叶子解释内容，职责不因为文件较短就合并回完整原生 HTML。首次阅读本页就应能判断实际路径、内容来源和运行边界。

我们 **正文 = [读取本页终端说明](引言.nodes(name="about"))**。
最后返回 *[选择本页的递归框架](布局.frame(body=`正文`, management=False))*。
