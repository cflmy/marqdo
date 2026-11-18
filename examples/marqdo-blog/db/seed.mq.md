---
markup: occlusion
title: 博客演示种子调用手册
import:
  document: lib/document.mq.md
---

本文件负责可调用的说明与编排，[seed.tables.md](seed.tables.md)负责可独立阅读的 Markdown 常量。调用时把 --fs-root 指向 examples/marqdo-blog，读取需要 fs.read；连接数据库或编译样式时还需要 plugin。

## posts

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

六篇演示文章覆盖文档、路由、原生集合、实时通信、会话和数据库。日期用于稳定排序，正文保持 Markdown 数据；这些常量不会自行写入数据库。

从站点根目录授予 fs.read 后调用。实际常量位于 [seed.tables.md](seed.tables.md#posts)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取posts的实际 Markdown 常量](document.read_table(path="db/seed.tables.md", title="posts"))*.

## tags

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

演示标签保留原始顺序与 slug，便于在独立空库中验证文章关联。读取这些行没有插入或删除效果；现有用户标签应由显式迁移管理。

从站点根目录授予 fs.read 后调用。实际常量位于 [seed.tables.md](seed.tables.md#tags)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取tags的实际 Markdown 常量](document.read_table(path="db/seed.tables.md", title="tags"))*.

## post_tags

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | 与原字面表一致的数据类型和记录/列式形态 |

关联行保持文章与标签的演示对应关系。它们适用于按同一顺序初始化的空库；已有数据需要重新映射 id，读取函数不会清空或重建任何表。

从站点根目录授予 fs.read 后调用。实际常量位于 [seed.tables.md](seed.tables.md#post_tags)；读取过程保留数值、布尔、空值和表格形态，并检查标题与行列结构。数据文件中的说明不会作为程序运行。

完成后返回*[读取post_tags的实际 Markdown 常量](document.read_table(path="db/seed.tables.md", title="post_tags"))*.
