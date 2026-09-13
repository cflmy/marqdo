## posts

帖子主表。`slug` 用于详情路径；`board` 为板块显示名，`board_id` 关联 `boards`。

`posts` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| title | text | false | |
| slug | text | false | 是 |
| summary | text | true | |
| body | text | true | |
| board | text | true | |
| board_id | integer | true | |
| author | text | true | |
| comments_count | integer | true | |
| views | integer | true | |
| created_at | timestamp | true | |
| updated_at | timestamp | true | |

*`posts`*

## news

新闻主表。`board_id` 关联 `news_boards`；`board` 为板块显示名。

`news` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| title | text | false | |
| slug | text | false | 是 |
| summary | text | true | |
| body | text | true | |
| board | text | true | |
| board_id | integer | true | |
| created_at | timestamp | true | |
| updated_at | timestamp | true | |

*`news`*

## boards

帖子板块。

`boards` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| name | text | false | |
| slug | text | false | 是 |

*`boards`*

## news_boards

新闻板块。

`news_boards` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| name | text | false | |
| slug | text | false | 是 |

*`news_boards`*

## notices

首页公告。

`notices` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| content | text | false |
| created_at | timestamp | true |

*`notices`*

## comments

评论（按目标类型 + slug 挂接）。

`comments` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| target_type | text | false |
| target_slug | text | false |
| body | text | false |
| author | text | true |
| created_at | timestamp | true |

*`comments`*

## chat_rooms

公聊房间。

`chat_rooms` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| slug | text | false | 是 |
| name | text | false | |
| summary | text | true | |

*`chat_rooms`*

## topics

专题占位。

`topics` =

| 字段 | 类型 | 可空 | 索引 |
|------|------|------|------|
| id | integer | false | |
| slug | text | false | 是 |
| title | text | false | |
| summary | text | true | |

*`topics`*

## api_keys

API Key 校验用哈希表（应用层 scopes）。

`api_keys` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| name | text | false |
| hash | text | false |
| scopes | text | true |

*`api_keys`*
