# 当前 Artifact 博客数据

这里保存原生字面 GFM 数据。调用函数通过 document.read_table 读取所选章节；不会执行本文件中的说明或示例。

## schema

当前站点的文章字段包括标题、slug、摘要、正文、标签和创建日期。原生数据库初始化器负责 id；本结构保留当前自动列表与详情页所需字段。

| 字段 | 类型 | 可空 |
|------|------|------|
| title | text | False |
| slug | text | False |
| summary | text | True |
| content | text | True |
| tag | text | True |
| created_at | text | True |

## seed

两篇当前站点的起始文章保留稳定 slug、内容与日期。只有调用 open 且文章表为空时才插入；读取种子本身不会改变现有文章。

| title | slug | summary | content | tag | created_at |
|-------|------|---------|---------|-----|------------|
| Hello Marqdo | hello | Welcome | Hello — executable documents. | intro | 2026-09-01 |
| Artifact Web | artifact-web | Document first | Pages are mq.md files. | design | 2026-09-28 |
