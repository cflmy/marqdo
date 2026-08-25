## 结构

`字段` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

**`字段`**

## 种子

`行` =

| id | title | body |
|----|-------|------|
| 1 | 欢迎来到网络扩展站 | 首页列表来自 SQLite。 |
| 2 | 登录门禁后台 | /admin 页面需要 session 会话。 |
| 3 | WebSocket 实时 | /live 端点会把你的消息实时回显。 |

**`行`**