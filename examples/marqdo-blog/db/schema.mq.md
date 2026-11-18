---
markup: occlusion
title: 博客表结构调用手册
import:
  document: lib/document.mq.md
---

本文件负责可调用的说明与编排，[schema.tables.md](schema.tables.md)负责可独立阅读的 Markdown 常量。调用时把 --fs-root 指向 examples/marqdo-blog，读取需要 fs.read；连接数据库或编译样式时还需要 plugin。

## posts

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

文章结构保留 id、标题、slug、摘要、Markdown 正文、主标签和时间字段。slug 对应文章详情路由；可空声明控制存储约束，路由授权仍由端点或门禁负责。

从站点根目录授予 fs.read 后调用。实际常量位于 [schema.tables.md](schema.tables.md#posts)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取posts的实际 Markdown 常量](document.read_table(path="db/schema.tables.md", title="posts"))*.

## tags

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

标签结构保留显示名称与稳定 slug。显示文字可以改动，路由和文章主标签应继续引用稳定标识；唯一索引需要在正式迁移中明确声明。

从站点根目录授予 fs.read 后调用。实际常量位于 [schema.tables.md](schema.tables.md#tags)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取tags的实际 Markdown 常量](document.read_table(path="db/schema.tables.md", title="tags"))*.

## post_tags

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

关联结构保留文章与标签标识。种子关系依赖空数据库中预期的插入顺序；向已有用户库导入时应先映射真实 id，不能直接复用演示序号。

从站点根目录授予 fs.read 后调用。实际常量位于 [schema.tables.md](schema.tables.md#post_tags)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取post_tags的实际 Markdown 常量](document.read_table(path="db/schema.tables.md", title="post_tags"))*.
