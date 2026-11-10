# 博客表结构

这里保存原生字面 GFM 数据。调用函数通过 document.read_table 读取所选章节；不会执行本文件中的说明或示例。

## posts

文章结构保留 id、标题、slug、摘要、Markdown 正文、主标签和时间字段。slug 对应文章详情路由；可空声明控制存储约束，路由授权仍由端点或门禁负责。

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

## tags

标签结构保留显示名称与稳定 slug。显示文字可以改动，路由和文章主标签应继续引用稳定标识；唯一索引需要在正式迁移中明确声明。

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| name | text | false |
| slug | text | false |

## post_tags

关联结构保留文章与标签标识。种子关系依赖空数据库中预期的插入顺序；向已有用户库导入时应先映射真实 id，不能直接复用演示序号。

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| post_id | integer | false |
| tag_id | integer | false |
