## posts

文章主表结构。`slug` 用于详情页 `/post/{slug}` 的动态路由；`content` 存 Markdown 正文；`tag` 是主标签（tags 表 slug）。

`schema` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| slug | text | false |
| summary | text | true |
| content | text | true |
| tag | text | true |
| created_at | text | true |
| updated_at | text | true |

**`schema`**

## tags

标签表结构。

`tags` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| name | text | false |
| slug | text | false |

**`tags`**

## post_tags

文章-标签关联表结构。

`post_tags` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| post_id | integer | false |
| tag_id | integer | false |

**`post_tags`**