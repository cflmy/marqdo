# Article data

Literal article schema and demonstration content. Reading these tables does not open a database or insert rows. Follow articles.mq.md for callable use; the database initializer decides whether seeding is appropriate.

## 结构

Field definitions for the native database initializer; nullable values remain booleans.

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

## 种子

Initial demonstration records in their original column-oriented shape; ids and stable slugs retain their meaning.

| id | title | body |
|----|-------|------|
| 1 | 欢迎来到网络扩展站 | 首页列表来自 SQLite。 |
| 2 | 登录门禁后台 | /admin 页面需要 session 会话。 |
| 3 | WebSocket 实时 | /live 端点会把你的消息实时回显。 |
