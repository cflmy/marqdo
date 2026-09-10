## posts

帖子主表。`slug` 用于 `/post/{slug}`；`board` 为板块名（简化外键为文本，便于 SQLite 演示）。

`posts` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| title | text | false | |
| slug | text | false | 是 |
| summary | text | true | |
| body | text | true | |
| board | text | true | |
| author | text | true | |
| created_at | timestamp | true | |
| updated_at | timestamp | true | |

**`posts`**

## news

新闻主表（与帖子对称，验收「news 模块可写」）。

`news` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| title | text | false | |
| slug | text | false | 是 |
| summary | text | true | |
| body | text | true | |
| created_at | timestamp | true | |
| updated_at | timestamp | true | |

**`news`**

## api_keys

API Key 校验用哈希表（应用层 scopes；校验原语见 `web_api_key_check`）。

`api_keys` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| name | text | false |
| hash | text | false |
| scopes | text | true |

**`api_keys`**
