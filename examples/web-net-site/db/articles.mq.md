## schema

`fields` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

**`fields`**

## seed

`rows` =

| id | title | body |
|----|-------|------|
| 1 | Welcome to the web-net site | Home page lists rows from SQLite. |
| 2 | Login-gated admin | The /admin page requires a session cookie. |
| 3 | WebSocket live | The /live endpoint echoes your message in real time. |

**`rows`**