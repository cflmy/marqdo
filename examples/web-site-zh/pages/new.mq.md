---
markup: occlusion
type: web
view: main
route: /new
title: 写文章 · Marqdo 中文站
import:
  布局: ../components/index.mq.md
  引言: ../components/intro.mq.md
  文章: ../components/articles.mq.md
  组件: ext/web/网页/组件/索引.mq.md
  表单: ext/data/表单.mq.md
  表: lib/表.mq.md
---

本 Document 显式声明 /new，介绍留在终端 Markdown，框架按职责共享。请求资源继续拥有自己的数据和提交责任。 页面启动函数不初始化数据库或开启另一个监听进程。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 共享框架内的本页内容与请求资源 |

装配授予 fs.read 以读取终端说明，构造描述仍是纯计算。服务器在请求期间为真实会话准备表单 CSRF 与返回路径，不把启动时令牌写死到页面。缺少挂载资源应保留诊断，不能显示固定成功的空表单。

结构保持稳定，请求资源观察当前数据与身份。父层决定位置，终端叶子解释内容，职责不因为文件较短就合并回完整原生 HTML。首次阅读本页就应能判断实际路径、内容来源和运行边界。

我们 **正文 = [读取本页终端说明](引言.nodes(name="new"))**。
我们 **引用 = [引用同一个实际挂载的表单](表单.引用(标识="article"))**、**操作 = [保留当前请求 CSRF 与提交上下文](组件.请求资源(值=`引用`, 槽="article-form"))** 和 **正文 = [说明位于实际操作之前](表.拼接(a=`正文`, b=`操作`))**。
最后返回 *[选择本页的递归框架](布局.frame(body=`正文`, management=False))*。
