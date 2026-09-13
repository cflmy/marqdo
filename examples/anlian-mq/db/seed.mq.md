## boards

帖子板块：日常 / 开发 / 聊天。

`boards` =

| @ | name | slug |
|---|------|------|
| 1 | 日常 | daily |
| 2 | 开发 | develop |
| 3 | 聊天 | chat |

*`boards`*

## news_boards

新闻板块：站务 / 技术。

`news_boards` =

| @ | name | slug |
|---|------|------|
| 1 | 站务 | site |
| 2 | 技术 | tech |

*`news_boards`*

## posts

种子帖子（标题/短链与原 anlian-mq 一致）。

`posts` =

| @ | title | slug | summary | body | board | board_id | author | comments_count | views |
|---|-------|------|---------|------|-------|----------|--------|----------------|-------|
| 1 | 欢迎来到暗恋见君 | welcome | 期待人们能够继续相信爱情。 | 这是用 Marqdo 重写的验收站点。页面、库表与门禁都用 GFM 表格装配——代码即文档。 | 日常 | 1 | 站务 | 2 | 42 |
| 2 | 如何用表格写论坛 | tables-for-forums | 导航、表单、种子数据都是可读表格。 | 若某功能逼你写一长串 json.set，请先审视 ext/web 是否缺命名方法。 | 开发 | 2 | 站务 | 0 | 18 |
| 3 | 公聊室已开放 | chat-open | WebSocket 广播房间。 | 打开 /chat，连上 /chat/ws，消息会扇出给同房间所有人。 | 聊天 | 3 | 站务 | 0 | 9 |

*`posts`*

## news

种子新闻。

`news` =

| @ | title | slug | summary | body | board | board_id |
|---|-------|------|---------|------|-------|----------|
| 1 | 站点迁往 Marqdo | migrate-to-marqdo | 原生层换 Go libweb，作者面保持表格。 | 暗恋见君的验收目标：登录、发帖、列表详情、聊天与站长工具都能用 .mq.md 写完。 | 站务 | 1 |
| 2 | 外接 PostgreSQL / Redis | pg-redis | 生产可用外接驱动。 | 本示例默认 sqlite:；把数据库地址换成 postgres:// 即可。 | 技术 | 2 |

*`news`*

## notices

首页公告（2–3 条）。

`notices` =

| @ | content | created_at |
|---|---------|------------|
| 1 | 欢迎来到暗恋见君论坛。本站支持匿名与实名交流，请友善发言。 | 2026-09-01 10:00:00 |
| 2 | 站点正用 Marqdo 表格重写验收：导航、首页四段、列表与公聊对齐原站 DOM。 | 2026-09-08 14:30:00 |
| 3 | 发帖与评论请遵守社区规范；站务公告将在此置顶。 | 2026-09-10 09:00:00 |

*`notices`*

## comments

欢迎帖下的评论。

`comments` =

| @ | target_type | target_slug | body | author | created_at |
|---|-------------|-------------|------|--------|------------|
| 1 | post | welcome | 终于等到用表格写论坛的一天。 | 路人甲 | 2026-09-10 11:00:00 |
| 2 | post | welcome | 期待人们能够继续相信爱情。 | 见君 | 2026-09-10 12:15:00 |

*`comments`*

## chat_rooms

公聊：大厅 + 闲聊。

`chat_rooms` =

| @ | slug | name | summary |
|---|------|------|---------|
| 1 | lobby | 大厅 | 全站公共聊天室。 |
| 2 | casual | 闲聊 | 轻松闲聊，勿刷屏。 |

*`chat_rooms`*

## topics

专题占位条目。

`topics` =

| @ | slug | title | summary |
|---|------|-------|---------|
| 1 | glass-ui | 粉玻璃界面 | 首页与列表的毛玻璃视觉专题（占位）。 |
| 2 | tables-as-code | 表格即代码 | Marqdo GFM 表装配专题（占位）。 |

*`topics`*
