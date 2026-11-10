---
markup: occlusion
type: web
view: main
route: /
title: 网络扩展站
import:
  布局: components/index.mq.md
  首页: components/home.mq.md
  数据库: db/index.mq.md
  表单定义: forms.mq.md
  主题: styles/shell.mq.md
  网页: ext/web/网页.mq.md
  表单: ext/data/表单.mq.md
  后台: ext/data/后台.mq.md
  鉴权: ext/security/鉴权.mq.md
  实时: ext/net/实时.mq.md
  文件: lib/文件.mq.md
  result: lib/result.mq.md
---

从本入口依次阅读组件入口、Markdown 内容叶子、数据库、表单资源与浏览器客户端。首页、关于、发布与工具页都是显式路由 Document；后台保留原生登录门禁，/live 单独归网络资源管理。页面不再依赖已移除的装配接口，启动也不会拼接整页 HTML。

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| 结果 | list | 共享递归布局内的介绍、实时控件与请求期间的文章列表 |

启动装配只读取终端 Markdown 内容，数据库查询保留为请求资源。发布后下一次访问就能看到新文章，无需重启或重新创建页面。客户端使用原生控件和官方桥；布局本身不初始化数据库或清除业务数据。编辑一块内容时沿组件导入找到对应叶子，不在首页复制导航、侧栏和页脚。

我们先 **正文 = [组合首页三个独立职责](首页.节点())**，随后返回 *[安放到共享的递归布局](布局.框架(正文=`正文`))*。

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-net-site-zh" | 相对仓库的显式站点发现目录 |
| port | number | 18084 | 本机回环监听端口 |
| url | any | None | None 使用站点根的 SQLite；验收指定独占临时 URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| 结果 | map | 保留文章、表单、鉴权与回显资源的阻塞监听结果 |

从仓库根使用`root`="examples/web-net-site-zh"、`port`=18084、`url`=None，授予 plugin,fs.write,network.write，并设置 --fs-root examples/web-net-site-zh。先通过开发手册把当前浏览器产物放进 public/generated。缺少运行时文件时停止，不呈现看似可用的空控件。

数据库只为空表播种，已有文章保留。主题由原始属性表编译后显式写到生成目录，再发现页面并按顺序启用后台、绑定鉴权、挂载公开文章表单、绑定回显。每一步保留新应用，最后才监听。本地 admin/secret 是演示夹具，真实部署必须替换账户来源与网络策略；公开回显只绑定回环地址。服务阻塞于当前进程，关闭由调用者负责。显式 url 用于隔离验收，SQLite 路径仍经过授权根检查；不能为测试删除示例原有 data 目录。主题是可再生文件，写入失败会阻止后续监听，单文件原子替换不意味着数据库和样式之间存在事务。

`产物` =

| 文件 |
|---|
| public/generated/marqdo_wasm.wasm |
| public/generated/marqdo-bridge.js |

- [在启动前确认实际运行时文件](for 产物 as 路径)
  1. not [运行时资源确实存在](文件.存在(路径=`路径`))
    返回 *[指出所需浏览器准备](result.error(code="net_site.browser_assets", message="请先把当前 WASM 与官方桥构建到 public/generated", operation="serve"))*。
我们 **存储 = [打开并保留文章数据库](数据库.打开(url=`url`))**，**样式 = [编译既有的声明式主题](主题.编译())**，并 **[建立明确的生成目录](文件.建目录(路径="public/generated"))**、**[原生写入本次主题](文件.原子写文本(路径="public/generated/theme.css", 内容=`样式`))**。
我们 **应用 = [发现已声明页面并保留站点上下文](网页.准备站点(根=`root`, 主机="127.0.0.1", 端口=`port`, 数据库=`存储`, 静态目录="public"))** 和 **应用 = [保留原生后台能力](后台.启用(应用=`应用`))**。

`用户` =

| @ | username | password | role |
|---|---|---|---|
| 1 | admin | secret | admin |

我们 **应用 = [绑定真实登录会话与后台门禁](鉴权.绑定(应用=`应用`, 用户=`用户`, 登录路径="/admin/login", 登录返回="/admin", 退出返回="/"))**，**文章 = [构造共享校验资源](表单定义.文章())** 和 **应用 = [明确挂载公开发布表单](表单.挂载(应用=`应用`, 标识="article", 资源=`文章`))**。
接着 **应用 = [绑定本地演示的公开回显资源](实时.绑定(应用=`应用`, 路径="/live", 模式="echo", 需要登录=False))**。
全部资源就绪后返回 *[在保留的站点根中开始监听](网页.监听(应用=`应用`))*。
