## schema

`fields` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| slug | text | false |
| summary | text | true |
| body | text | true |

*`fields`*

## seed

卡片显示 summary；/post/{slug} 显示完整 body（Markdown）。

`rows` =

| id | title | slug | summary | body |
|----|-------|------|---------|------|
| 1 | 你好 Marqdo | hello-marqdo | 一份可执行 Markdown 既是站点说明，也是上线制品——路由、表单与样式都用表声明。 | "Marqdo 把 .mq.md 当作可执行文档。编辑器里读到的叙述，就是服务器跑起来的同一份制品：GFM 表变成 schema、表单、导航与 CSS。本 Demo 走完这条契约——打开「写文章」、触发规则表校验，再发布一行，首页与后台同时出现，全程无需作者业务 JavaScript。" |
| 2 | 表即全栈 | tables-full-stack | Schema、表单、路由、后台与主题令牌共用一套表方言，而不是 JS/CSS 分家。 | "常见技术栈把关切拆进多种语言：数据用 SQL/ORM，输入用表单库，URL 用路由，外观用手写样式。Marqdo 把这些表面收成作者本来就会写的 GFM 表。字段表、规则表、绑定表、样式表不是比喻，而是运行时。结果是一份既能当散文扫读、又能当程序执行的站点。" |
| 3 | 零作者 JS 与 CSS | zero-author-js-css | 业务留在 Markdown；外观是样式表加宿主壳——无作者业务 JS、无作者 .css。 | "作者业务 JavaScript 从构造上为零：校验来自服务端规则表，列表/详情由绑定表交给宿主渲染。主题令牌写在属性/值表里，经样式装配变成 CSS，作者也不必维护 .css 文件。本 Demo 关闭壳样式，只让声明过的样式上色——方便会议观众当场核对主张。" |
| 4 | 一表打通卡片与详情 | card-to-detail | 列表把 href 绑到 slug；/post/{slug} 加详情=真即可打开全文。 | "点击首页任意卡片。列表绑定用 articles.slug 作为 href，链接前缀补上 /post/。详情页复用同一张表，查询条件 slug = {slug}，再页面.详情，首行就渲染成完整文章（标题 + Markdown 正文），而不是卡片网格。这与求道量子等更大的 Marqdo 站同一套路。" |
| 5 | 样式表不是样式文件 | style-tables | 具名样式表（kicker、intro_title、card_title）通过绑定表第三列挂上。 | "引言、卡片与壳层都用属性/值/样式绑定。样式列指向 styles/shell.mq.md 里的具名属性表。引言装配或主体装配时，这些表变成页面上的 CSS 类。改 Demo 外观不必打开 .css——改可读的 Markdown 表，文案与样式仍挨在一起。" |
| 6 | 表单来自字段表与规则表 | forms-from-tables | 「写文章」空标题会因规则表失败；合法提交写入与后台共享的一行。 | "写文章与后台挂载同一表单对象：字段表声明标签与类型，规则表声明 required、max 与提示文案。空标题提交时错误来自服务端，不是客户端脚本。填好标题与 slug 再提交，卡片网格与后台列表都会出现——一张 schema、两个表面、零作者 JS。" |
| 7 | 种子数据就是文档 | seed-is-docs | 这些段落写在 GFM 种子表里——Demo 正文也是程序的一部分。 | "种子行与 schema 同在 db/articles.mq.md。首次启动若表为空，宿主会插入它们。于是 Demo 叙述与代码同版本：卡片用稍长摘要，详情用更长正文，slug 保证深链稳定。审稿人打开种子表，就能看到会场站点将展示的内容。" |
| 8 | 为什么要做这个 Demo | why-demo-matters | WWW Demo 需要审稿人几分钟内能跑、能点、能对照源码的制品。 | "Demo 轨道要的是可运行系统，不是只靠幻灯片。本样站刻意很小：一张 SQLite 表、几条路由、一套文学风壳层——却覆盖论文主张（表即全栈），并配三步操作脚本。克隆仓库，运行样站，点开卡片，弄坏一次表单，再把 index.mq.md 与浏览器对照。这个闭环就是投稿本身。" |

*`rows`*
