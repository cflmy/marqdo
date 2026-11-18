# Article data

Literal article schema and demonstration content. Reading these tables does not open a database or insert rows. Follow articles.mq.md for callable use; the database initializer decides whether seeding is appropriate.

## schema

Field definitions for the native database initializer; nullable values remain booleans.

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

## seed

Initial demonstration records in their original column-oriented shape; ids and stable slugs retain their meaning.

| id | title | body |
|----|-------|------|
| 1 | Welcome to the web-net site | Home page lists rows from SQLite. |
| 2 | Login-gated admin | The /admin page requires a session cookie. |
| 3 | WebSocket live | The /live endpoint echoes your message in real time. |
