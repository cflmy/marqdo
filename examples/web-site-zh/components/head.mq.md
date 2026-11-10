---
markup: occlusion
title: 首页 SEO、共享字体与主题声明
import:
  组件: ext/web/网页/组件/索引.mq.md
  页头: ext/web/页头.mq.md
---

本叶子保留原站的首页元数据与 Google Fonts 资源地址。装配只声明链接，不下载字体；主题由服务入口从表编译并写到明确静态目录。

## nodes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| homepage | bool | False | 仅首页显式保留原来的首页 SEO |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | 进入真正 head 的纯 SEO、字体与主题资源 |

使用`homepage`=False，在装配阶段纯构造。元数据为 key/value 行记录，字体与样式继续使用 rel/href/crossorigin 原生字段。只有首页显式选择原来的 SEO，其余页面只共享字体与主题，避免把首页描述错误地复制到每篇文章。原生渲染将它们收集到真正 head，去重相同链接，不在正文或片段中输出占位。

字体是否成功送达取决于浏览器能否访问外部服务；不可用时仍使用主题声明的后备字体。地址声明不等于生成文件，服务入口须在监听之前编译并写出确切路径。资源构造不加载插件、不验证远端状态、不运行脚本，也不把网络错误当作已经加载成功。

字体表中的第一项预连接样式服务，第二项保留字体源的 anonymous 跨域设置，第三项保留原始字体集合与字重。最后的站内样式链接只对应本次生成产物，源头仍是可读属性表。标题与文案保留在各自 Document，页头不承担整页配置或应用变异。离线预览使用同一个元数据收集过程，片段只更新已打开页面，不重复注入字体与 SEO。

`元数据` =

| @ | key | value |
|---|---|---|
| 1 | description | 表即全栈 — Marqdo 网页示例；作者零业务 JavaScript。 |
| 2 | og:type | website |
| 3 | og:title | Marqdo 中文站 |

`链接` =

| @ | rel | href | crossorigin |
|---|---|---|---|
| 1 | preconnect | https://fonts.googleapis.com | |
| 2 | preconnect | https://fonts.gstatic.com | anonymous |
| 3 | stylesheet | https://fonts.googleapis.com/css2?family=Great+Vibes&family=Noto+Sans+SC:wght@300;400;500;700&family=Noto+Serif+SC:wght@400;600&display=swap | |
| 4 | stylesheet | /static/generated/theme.css | |

我们 **选定元数据 = None**，其他页面只保留共享链接。
1. `homepage`
  我们 **选定元数据 = 元数据**，只给首页保留原 SEO。
我们 **资源 = [保留实际元数据与原生资源链接](页头.声明(元数据=`选定元数据`, 链接=`链接`))**。
最后返回 *[把纯页头声明与结构一起保留](组件.请求资源(值=`资源`, 槽="document-head"))*。
