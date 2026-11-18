---
markup: occlusion
type: web
view: main
route: /
title: Marqdo 中文站
import:
  布局: components/index.mq.md
  引言: components/intro.mq.md
  文章: components/articles.mq.md
  数据库: db/index.mq.md
  表单定义: forms.mq.md
  主题: styles/index.mq.md
  网页: ext/web/网页.mq.md
  表单: ext/data/表单.mq.md
  鉴权: ext/security/鉴权.mq.md
  文件: lib/文件.mq.md
  表: lib/表.mq.md
---

本入口是可执行建站手册。先读组件入口的公共与后台结构，再读终端 Markdown、请求期间文章、表单和数据库。首页、关于、写文章、后台与短链详情各自声明路由。当前会话决定导航，实际访问策略由安全资源执行；自定义后台不启用另一套原生后台首页。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 公共框架内的 Markdown 介绍与当前文章卡片 |

启动装配只读取内容叶子并构造查询描述，不初始化或清空数据库。监听应用保留的实际资源在请求期间查询文章；发布后下次访问直接看到新记录。共享布局保留导航、侧栏、页脚、首页 SEO、字体与原始文学主题。改一块内容沿导入找到自己的叶子，不能把完整页面壳复制到每个路由。

我们 **说明 = [读取首页终端介绍](引言.nodes(name="home"))**、**卡片 = [描述当前文章卡片](文章.cards())** 和 **正文 = [让说明先于业务记录](表.拼接(a=`说明`, b=`卡片`))**。
最后返回 *[放入共享公共框架并保留首页 SEO](布局.frame(body=`正文`, homepage=True))*。

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-site-zh" | 相对仓库的明确站点发现目录 |
| port | number | 18082 | 本机回环监听端口 |
| url | any | None | None 保留站点根 SQLite；验收指定独占临时 URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | 数据、表单、鉴权及自定义后台门禁齐备后的阻塞监听结果 |

从仓库根使用`root`="examples/web-site-zh"、`port`=18082、`url`=None，授予 plugin,fs.write,network.write 并设置 --fs-root examples/web-site-zh。初始化只为空表播种，已有文章全部保留。原始主题数据叶子不变，样式入口只把旧壳选择器映射到明确的 View 结构，并恢复原来由装配操作附着的命名属性表。

主题在发现页面之前编译并原生写入 public/generated。准备站点不监听；同一文章表单同时供公共发布页与后台引用。随后绑定本地演示会话，并对实际 /admin 路径施加角色门禁，每次更新都保留新应用，最后才监听。admin/demo 只是回环演示账户，部署应替换用户来源与网络策略。服务阻塞当前进程，由调用者负责停止；不要为验收删除示例已有 data 目录。

我们 **存储 = [初始化并保留原有文章](数据库.open(url=`url`))**、**样式 = [编译真实 View 主题映射](主题.css())**，并 **[建立明确的生成目录](文件.建目录(路径="public/generated"))**、**[原生写入本次编译的主题](文件.原子写文本(路径="public/generated/theme.css", 内容=`样式`))**。
接着 **应用 = [发现五个明确声明的页面](网页.准备站点(根=`root`, 主机="127.0.0.1", 端口=`port`, 数据库=`存储`, 静态目录="public"))**、**资源 = [构造共享服务器校验规则](表单定义.article())** 和 **应用 = [明确挂载公开文章表单](表单.挂载(应用=`应用`, 标识="article", 资源=`资源`))**。

`用户` =

| @ | username | password | role |
|---|---|---|---|
| 1 | admin | demo | admin |

我们 **应用 = [绑定实际登录会话](鉴权.绑定(应用=`应用`, 用户=`用户`, 登录路径="/login", 登录返回="/admin", 退出返回="/"))** 和 **应用 = [保护直接访问自定义后台的请求](鉴权.门禁(应用=`应用`, 路径="/admin", 角色="admin", 匹配="prefix", 拒绝="redirect", 排除="/login"))**。
资源与策略就绪后返回 *[开始实际监听](网页.监听(应用=`应用`))*。
