## posts

种子帖子（`@` 行记录表）。

`posts` =

| @ | title | slug | summary | body | board | author |
|---|-------|------|---------|------|-------|--------|
| 1 | 欢迎来到暗恋见君 | welcome | 期待人们能够继续相信爱情。 | 这是用 Marqdo 重写的验收站点。页面、库表与门禁都用 GFM 表格装配——代码即文档。 | 日常 | 站务 |
| 2 | 如何用表格写论坛 | tables-for-forums | 导航、表单、种子数据都是可读表格。 | 若某功能逼你写一长串 json.set，请先审视 ext/web 是否缺命名方法。 | 开发 | 站务 |
| 3 | 公聊室已开放 | chat-open | WebSocket 广播房间。 | 打开 /chat，连上 /chat/ws，消息会扇出给同房间所有人。 | 聊天 | 站务 |

**`posts`**

## news

`news` =

| @ | title | slug | summary | body |
|---|-------|------|---------|------|
| 1 | 站点迁往 Marqdo | migrate-to-marqdo | 原生层换 Go libweb，作者面保持表格。 | 暗恋见君的验收目标：登录、发帖、列表详情、聊天与站长工具都能用 .mq.md 写完。 |
| 2 | 外接 PostgreSQL / Redis | pg-redis | 生产可用外接驱动。 | 本示例默认 sqlite:；把数据库地址换成 postgres:// 即可。 |

**`news`**
