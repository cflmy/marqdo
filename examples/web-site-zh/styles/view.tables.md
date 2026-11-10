# View 几何适配

这些字面声明只适配明确的 View 包装。原始色板、动画、排版和组件属性保留在 shell.tables.md；可调用入口说明如何将递归框架连接到这些数据。

## geometry

基础预览层继承本站字体与全宽。资源仍保留结构值，其语义子块参与原来的命名网格；列表使用 Markdown，不增加私有引言方言。

| selector | property | value |
|---|---|---|
| .mq-document .mq-view | max-width | none |
| .mq-document .mq-view | margin | 0 |
| .mq-document .mq-view | padding | 0 |
| .mq-document .mq-view | line-height | inherit |
| .mq-document .mq-view | font-family | inherit |
| .mq-document .site-layout | display | grid |
| .mq-document .site-layout | grid-template-rows | auto 1fr auto |
| .mq-document .site-layout | min-height | 100vh |
| .mq-document .site-layout > .slot-nav | display | contents |
| .mq-document .site-layout > .slot-sidebar | display | contents |
| .mq-document .site-layout .main-intro ul | padding | 0 |
| .mq-document .site-layout .main-intro ul | list-style | none |
| .mq-document .site-layout .main-intro li > p | margin | 0 |
| .mq-document .site-layout .main-intro blockquote > p | margin | 0 |

## selectors

只有这些结构地址改变，原属性、值、媒体与顺序保留。这是本站迁移表，不是新的页面语言。

| @ | old | new |
|---|---|---|
| 1 | body.has-sidebar | .site-layout.has-sidebar |
| 2 | body.no-sidebar | .site-layout.no-sidebar |
| 3 | body.layout-stacked | .site-layout.layout-stacked |
| 4 | body.is-desk | .site-layout.is-desk |
| 5 | main.main | section.main |
| 6 | .main-intro > .claim | .main-intro ul |
| 7 | .main-intro > .steps | .main-intro ol |
| 8 | .intro_title | .main-intro h1 |
| 9 | .kicker | .main-intro blockquote |

## areas

按同一级联顺序编译这些原始章节。

| name |
|---|
| vars |
| motion |
| layout |
| responsive |
| chrome |
| widgets |


## properties

在实际语义目标上恢复每组命名属性，原生编译器给名称补上开头的类选择器点。

| @ | name | selector |
|---|---|---|
| 1 | topnav | mq-document .site-layout .topnav |
| 2 | side_panel | mq-document .site-layout .side |
| 3 | footer | mq-document .site-layout .foot |
| 4 | card_title | mq-document .site-layout .content.cards h2 |
| 5 | card_body | mq-document .site-layout .content.cards p |
| 6 | kicker | mq-document .site-layout .main-intro blockquote |
| 7 | intro_title | mq-document .site-layout .main-intro h1 |
| 8 | lede | mq-document .site-layout .main-intro .slot-intro-content > p |
| 9 | claim | mq-document .site-layout .main-intro ul > li |
| 10 | step | mq-document .site-layout .main-intro ol > li |
